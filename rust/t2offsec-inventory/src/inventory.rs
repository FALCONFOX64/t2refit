// SPDX-License-Identifier: Apache-2.0

//! Read-only install and version inventory.
//!
//! Version probes run only the catalog's dash-flags, and only for binaries
//! that are not on the deny list. Nothing here opens a network connection
//! on its own.

use crate::catalog::Tool;
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
    if binary.is_empty() || args.is_empty() || args.len() > 4 {
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
                    path: found.map(|path| path.display().to_string()).unwrap_or_default(),
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
            path: found.map(|path| path.display().to_string()).unwrap_or_default(),
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
        if name.is_empty() {
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
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    fn rpm_versions(&self, packages: &[String]) -> Option<HashMap<String, String>> {
        if packages.is_empty() {
            return Some(HashMap::new());
        }
        if self.find_binary("rpm").is_none() {
            return None;
        }
        let mut command = Command::new("rpm");
        command
            .arg("-q")
            .arg("--qf")
            .arg("%{NAME}\\t%{VERSION}-%{RELEASE}\\n")
            .args(packages)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let output = command.output().ok()?;
        let mut found = HashMap::new();
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some((name, version)) = line.split_once('\t') {
                if !name.is_empty() && !version.is_empty() && !line.contains("not installed") {
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
        let script = format!(
            "import importlib.metadata as m\nprint(m.version(\"{distribution}\"))"
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let line = first_line(&String::from_utf8_lossy(&output.stdout));
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }

    fn command_version(&self, path: &Path, args: &[String]) -> Option<String> {
        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("");
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

fn run_version(path: &Path, args: &[String]) -> Option<String> {
    let mut child = Command::new(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let mut stderr = child.stderr.take()?;
    let out_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let err_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    });
    let start = Instant::now();
    let finished = loop {
        match child.try_wait() {
            Ok(Some(_)) => break true,
            Ok(None) if start.elapsed() > Duration::from_secs(2) => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => break false,
        }
    };
    let out = out_thread.join().unwrap_or_default();
    let err = err_thread.join().unwrap_or_default();
    if !finished {
        return None;
    }
    let line = first_line(&String::from_utf8_lossy(&out));
    if !line.is_empty() {
        return Some(line);
    }
    let err_line = first_line(&String::from_utf8_lossy(&err));
    if err_line.is_empty() {
        None
    } else {
        Some(err_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{load, Tool};
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
                ("proxychains4".to_string(), PathBuf::from("/usr/bin/proxychains4")),
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
        }
    }
}
