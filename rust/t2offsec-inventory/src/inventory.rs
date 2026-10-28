// SPDX-License-Identifier: Apache-2.0

//! Read-only install and version inventory.
//!
//! Version probes run only the catalog's dash-flags, and only for binaries
//! that are not on the deny list. Nothing here opens a network connection
//! on its own.

use crate::catalog::{valid_token, Tool};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const NEVER_EXEC: &[&str] = &[
    "proxychains",
    "proxychains4",
    "proxychains4-daemon",
    "airodump-ng",
    "aireplay-ng",
    "airmon-ng",
    "airbase-ng",
    "airtun-ng",
    "reaver",
    "wash",
    "kismet",
    "responder",
    "bettercap",
    "msfconsole",
    "msfvenom",
    "msfdb",
    "searchsploit",
    "hackrf_info",
    "hackrf_transfer",
    "hackrf_sweep",
    "rtl_test",
    "rtl_sdr",
    "rtl_tcp",
    "rtl_fm",
    "proxmark3",
    "pm3",
    "ubertooth-util",
    "ubertooth-rx",
    "ubertooth-scan",
    "setoolkit",
    "beef",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub group: String,
    pub present: bool,
    pub path: String,
    pub rpm_version: String,
    pub tool_version: String,
    pub note: String,
}

pub trait Lookup {
    fn find_binary(&self, name: &str) -> Option<PathBuf>;
    /// None means `rpm` itself is not installed. Missing packages are absent keys.
    fn rpm_versions(&self, packages: &[String]) -> Option<HashMap<String, String>>;
    fn python_version(&self, distribution: &str) -> Option<String>;
    fn command_version(&self, path: &Path, args: &[String]) -> Option<String>;
}

pub fn version_probe_allowed(binary: &str, args: &[String]) -> bool {
    if !valid_token(binary) || args.is_empty() || args.len() > 4 {
        return false;
    }
    if NEVER_EXEC.iter().any(|name| *name == binary) {
        return false;
    }
    args.iter()
        .all(|arg| arg.starts_with('-') && !arg.chars().any(char::is_whitespace))
}

pub fn inventory(tools: &[Tool], lookup: &dyn Lookup) -> Vec<Row> {
    let packages: Vec<String> = tools
        .iter()
        .filter(|tool| !tool.fedora_package.is_empty())
        .map(|tool| tool.fedora_package.clone())
        .collect();
    let rpms = lookup.rpm_versions(&packages);
    let mut rows = Vec::with_capacity(tools.len());
    for tool in tools {
        let found = if tool.binary.is_empty() {
            None
        } else {
            lookup.find_binary(&tool.binary)
        };
        let rpm_version = rpms
            .as_ref()
            .and_then(|map| map.get(&tool.fedora_package))
            .cloned()
            .unwrap_or_default();
        let python_version = if tool.python_distribution.is_empty() {
            None
        } else {
            lookup.python_version(&tool.python_distribution)
        };
        let mut note = String::new();
        let tool_version = if let Some(path) = found.as_ref() {
            if version_probe_allowed(&tool.binary, &tool.version_args) {
                lookup
                    .command_version(path, &tool.version_args)
                    .unwrap_or_default()
            } else if !tool.version_args.is_empty() {
                note = "version probe skipped".to_string();
                String::new()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let present = found.is_some() || !rpm_version.is_empty() || python_version.is_some();
        if let Some(version) = python_version {
            if tool_version.is_empty() {
                rows.push(Row {
                    id: tool.id.clone(),
                    group: tool.group.clone(),
                    present,
                    path: found
                        .map(|path| path.display().to_string())
                        .unwrap_or_default(),
                    rpm_version,
                    tool_version: version,
                    note,
                });
                continue;
            }
        }
        rows.push(Row {
            id: tool.id.clone(),
            group: tool.group.clone(),
            present,
            path: found
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            rpm_version,
            tool_version,
            note,
        });
    }
    rows
}

pub struct SystemLookup;

impl Lookup for SystemLookup {
    fn find_binary(&self, name: &str) -> Option<PathBuf> {
        if !valid_token(name) {
            return None;
        }
        let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|value| std::env::split_paths(&value).collect())
            .unwrap_or_default();
        for extra in ["/usr/bin", "/usr/sbin", "/bin", "/sbin"] {
            let path = PathBuf::from(extra);
            if !dirs.iter().any(|dir| dir == &path) {
                dirs.push(path);
            }
        }
        for dir in dirs {
            let candidate = dir.join(name);
            if candidate.parent().as_ref() != Some(&dir.as_path()) {
                continue;
            }
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    fn rpm_versions(&self, packages: &[String]) -> Option<HashMap<String, String>> {
        let mut names: Vec<&str> = Vec::new();
        for package in packages {
            if valid_token(package) && !names.contains(&package.as_str()) {
                names.push(package.as_str());
            }
        }
        if names.is_empty() {
            return Some(HashMap::new());
        }
        self.find_binary("rpm")?;
        let mut command = Command::new("rpm");
        command
            .arg("-q")
            .arg("--qf")
            .arg("%{NAME}\\t%{VERSION}-%{RELEASE}\\n")
            .arg("--")
            .args(&names);
        let captured = capture(&mut command, Duration::from_secs(15), 256 * 1024)?;
        if captured.timed_out {
            return None;
        }
        let mut found = HashMap::new();
        let text = String::from_utf8_lossy(&captured.stdout);
        for line in text.lines() {
            if let Some((name, version)) = line.split_once('\t') {
                if valid_token(name) && !version.is_empty() && !line.contains("not installed") {
                    found.insert(name.to_string(), version.trim().to_string());
                }
            }
        }
        Some(found)
    }

    fn python_version(&self, distribution: &str) -> Option<String> {
        if !is_distribution_name(distribution) {
            return None;
        }
        let script =
            format!("import importlib.metadata as m\nprint(m.version(\"{distribution}\"))");
        let mut command = Command::new("python3");
        command.arg("-c").arg(script);
        let captured = capture(&mut command, Duration::from_secs(2), 4096)?;
        if captured.timed_out {
            return None;
        }
        let line = first_line(&String::from_utf8_lossy(&captured.stdout));
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }

    fn command_version(&self, path: &Path, args: &[String]) -> Option<String> {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !version_probe_allowed(name, args) {
            return None;
        }
        run_version(path, args)
    }
}

fn is_distribution_name(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    value.len() <= 80
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-')
}

fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .chars()
        .filter(|ch| !ch.is_control())
        .take(160)
        .collect()
}

struct Captured {
    timed_out: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn read_capped(mut reader: impl Read, limit: usize) -> Vec<u8> {
    let mut buf = vec![0u8; limit];
    let mut total = 0;
    while total < limit {
        match reader.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    buf.truncate(total);
    buf
}

fn capture(command: &mut Command, timeout: Duration, cap: usize) -> Option<Captured> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().ok()?;
    let stdout = child.stdout.take()?;
    let stderr = child.stderr.take()?;
    let out_thread = thread::spawn(move || read_capped(stdout, cap));
    let err_thread = thread::spawn(move || read_capped(stderr, cap));
    let start = Instant::now();
    let timed_out = loop {
        match child.try_wait() {
            Ok(Some(_)) => break false,
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                break true;
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break true;
            }
        }
    };
    Some(Captured {
        timed_out,
        stdout: out_thread.join().unwrap_or_default(),
        stderr: err_thread.join().unwrap_or_default(),
    })
}

fn run_version(path: &Path, args: &[String]) -> Option<String> {
    let mut command = Command::new(path);
    command.args(args);
    let captured = capture(&mut command, Duration::from_secs(2), 4096)?;
    if captured.timed_out {
        return None;
    }
    let line = first_line(&String::from_utf8_lossy(&captured.stdout));
    if !line.is_empty() {
        return Some(line);
    }
    let err_line = first_line(&String::from_utf8_lossy(&captured.stderr));
    if err_line.is_empty() {
        None
    } else {
        Some(err_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{load, valid_token, Tool};
    use std::cell::RefCell;
    use std::path::PathBuf;

    struct Fake {
        calls: RefCell<Vec<Vec<String>>>,
        binaries: HashMap<String, PathBuf>,
    }

    impl Lookup for Fake {
        fn find_binary(&self, name: &str) -> Option<PathBuf> {
            self.binaries.get(name).cloned()
        }
        fn rpm_versions(&self, _packages: &[String]) -> Option<HashMap<String, String>> {
            Some(HashMap::new())
        }
        fn python_version(&self, distribution: &str) -> Option<String> {
            if distribution == "impacket" {
                Some("0.12.0".to_string())
            } else {
                None
            }
        }
        fn command_version(&self, _path: &Path, args: &[String]) -> Option<String> {
            self.calls.borrow_mut().push(args.to_vec());
            Some("unit-version".to_string())
        }
    }

    fn tool(id: &str, binary: &str, args: &[&str]) -> Tool {
        Tool {
            id: id.to_string(),
            group: "recon".to_string(),
            name: id.to_string(),
            purpose: "test".to_string(),
            upstream: "https://example.com".to_string(),
            license: "Apache-2.0".to_string(),
            fedora: "official".to_string(),
            fedora_package: String::new(),
            fedora_note: String::new(),
            copr: String::new(),
            binary: binary.to_string(),
            version_args: args.iter().map(|item| (*item).to_string()).collect(),
            python_distribution: String::new(),
            also_in: Vec::new(),
        }
    }

    #[test]
    fn runs_only_allowed_version_flags() {
        let fake = Fake {
            calls: RefCell::new(Vec::new()),
            binaries: HashMap::from([
                ("nmap".to_string(), PathBuf::from("/usr/bin/nmap")),
                (
                    "proxychains4".to_string(),
                    PathBuf::from("/usr/bin/proxychains4"),
                ),
                ("john".to_string(), PathBuf::from("/usr/bin/john")),
            ]),
        };
        let tools = vec![
            tool("nmap", "nmap", &["--version"]),
            tool("proxychains-ng", "proxychains4", &["--version"]),
            tool("john", "john", &["password"]),
        ];
        let rows = inventory(&tools, &fake);
        assert!(rows[0].present);
        assert_eq!(rows[0].tool_version, "unit-version");
        assert!(rows[1].present);
        assert_eq!(rows[1].tool_version, "");
        assert_eq!(rows[1].note, "version probe skipped");
        assert_eq!(rows[2].tool_version, "");
        assert_eq!(rows[2].note, "version probe skipped");
        let calls = fake.calls.borrow().clone();
        assert_eq!(calls, vec![vec!["--version".to_string()]]);
    }

    #[test]
    fn catalog_version_flags_are_safe() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../catalog/tools.toml");
        let catalog = load(&path).expect("catalog");
        for tool in catalog.tools {
            if tool.version_args.is_empty() {
                continue;
            }
            assert!(
                version_probe_allowed(&tool.binary, &tool.version_args),
                "{} would not be probed",
                tool.id
            );
            assert!(
                valid_token(&tool.binary),
                "{} binary is not a single name",
                tool.id
            );
            if !tool.fedora_package.is_empty() {
                assert!(
                    valid_token(&tool.fedora_package),
                    "{} package is not a single name",
                    tool.id
                );
            }
        }
    }

    #[test]
    fn slash_binary_is_not_probed_or_found() {
        assert!(!version_probe_allowed(
            "../nmap",
            &["--version".to_string()]
        ));
        assert!(!valid_token("a/b"));
        assert!(!valid_token("a\\b"));
        let lookup = SystemLookup;
        assert!(lookup.find_binary("../sh").is_none());
        assert!(lookup.find_binary("a/b").is_none());
    }

    #[test]
    fn version_probe_times_out_and_caps_output() {
        let sleep = Path::new("/bin/sleep");
        if sleep.is_file() {
            let start = Instant::now();
            assert!(run_version(sleep, &["30".to_string()]).is_none());
            assert!(start.elapsed() < Duration::from_secs(8));
        }
        let python = Path::new("/usr/bin/python3");
        if python.is_file() {
            let line = run_version(python, &["-c".to_string(), "print('A'*20000)".to_string()])
                .expect("python output");
            assert!(line.len() <= 160);
            assert!(line.starts_with('A'));
        }
    }
}
