// SPDX-License-Identifier: Apache-2.0

//! Load catalog/tools.toml. Parsing does not touch the network.
//!
//! Several fields are rendered by the Python CLI and the Markdown catalog.
//! This binary only needs ids, packages, and version flags.
#![allow(dead_code)]

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Installed catalog from the raccoonix-t2 RPM. Not a parent-directory search.
pub const INSTALLED_CATALOG: &str = "/usr/share/raccoonix-t2/catalog/tools.toml";

/// A single command or RPM name: no slash, backslash, or NUL.
pub fn valid_token(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    value.len() <= 200
        && !value.contains('/')
        && !value.contains('\\')
        && !value.contains('\0')
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '+'))
}

#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
    pub fedora_release: String,
    pub checked: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Group {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub kali_metapackages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tool {
    pub id: String,
    pub group: String,
    pub name: String,
    pub purpose: String,
    pub upstream: String,
    pub license: String,
    pub fedora: String,
    #[serde(default)]
    pub fedora_package: String,
    #[serde(default)]
    pub fedora_note: String,
    #[serde(default)]
    pub copr: String,
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub version_args: Vec<String>,
    #[serde(default)]
    pub python_distribution: String,
    #[serde(default)]
    pub also_in: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub meta: Meta,
    pub groups: Vec<Group>,
    pub tools: Vec<Tool>,
}

impl Catalog {
    pub fn select(&self, group: Option<&str>, tools: &[String]) -> Result<Vec<Tool>, String> {
        if let Some(group_id) = group {
            if !self.groups.iter().any(|item| item.id == group_id) {
                return Err(format!("unknown group {group_id}"));
            }
        }
        let mut chosen: Vec<Tool> = self
            .tools
            .iter()
            .filter(|tool| match group {
                None => true,
                Some(group_id) => {
                    tool.group == group_id || tool.also_in.iter().any(|item| item == group_id)
                }
            })
            .cloned()
            .collect();
        let mut wanted: Vec<&str> = Vec::new();
        for tool_id in tools {
            if !wanted.contains(&tool_id.as_str()) {
                wanted.push(tool_id.as_str());
            }
        }
        if !wanted.is_empty() {
            for tool_id in &wanted {
                if !self.tools.iter().any(|tool| tool.id == *tool_id) {
                    return Err(format!("unknown tool {tool_id}"));
                }
            }
            chosen.retain(|tool| wanted.contains(&tool.id.as_str()));
            if chosen.len() != wanted.len() {
                return Err("one of the tools is not in that group".to_string());
            }
        }
        Ok(chosen)
    }
}

pub fn load(path: &Path) -> Result<Catalog, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let catalog: Catalog =
        toml::from_str(&text).map_err(|err| format!("parse {}: {err}", path.display()))?;
    for tool in &catalog.tools {
        if !tool.binary.is_empty() && !valid_token(&tool.binary) {
            return Err(format!(
                "{} binary {:?} must be a single command name",
                tool.id, tool.binary
            ));
        }
        if !tool.fedora_package.is_empty() && !valid_token(&tool.fedora_package) {
            return Err(format!(
                "{} fedora_package {:?} must be a single package name",
                tool.id, tool.fedora_package
            ));
        }
    }
    Ok(catalog)
}

/// Catalog lookup. `explicit`, then `T2OFFSEC_CATALOG`, then the installed
/// path, then `catalog/tools.toml` in the current directory. Callers pass
/// that current-directory path in; this function does not walk parents.
pub fn resolve_catalog(
    explicit: Option<&str>,
    env_value: Option<&str>,
    installed: &Path,
    cwd_catalog: Option<&Path>,
) -> Result<PathBuf, String> {
    if let Some(value) = explicit {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("catalog not found: {}", path.display()));
    }
    if let Some(value) = env_value {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "T2OFFSEC_CATALOG is not a file: {}",
            path.display()
        ));
    }
    if installed.is_file() {
        return Ok(installed.to_path_buf());
    }
    if let Some(path) = cwd_catalog {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
    }
    Err(
        "catalog/tools.toml not found; pass --catalog, set T2OFFSEC_CATALOG, install raccoonix-t2, or run from the t2refit repository root"
            .to_string(),
    )
}

pub fn find_catalog(explicit: Option<&str>) -> Result<PathBuf, String> {
    let env_value = match std::env::var("T2OFFSEC_CATALOG") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(err) => return Err(format!("T2OFFSEC_CATALOG: {err}")),
    };
    let cwd_catalog = std::env::current_dir()
        .ok()
        .map(|dir| dir.join("catalog").join("tools.toml"));
    resolve_catalog(
        explicit,
        env_value.as_deref(),
        Path::new(INSTALLED_CATALOG),
        cwd_catalog.as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_catalog_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/tools.toml");
        let catalog = load(&path).expect("catalog");
        assert!(catalog.tools.len() > 80);
        assert!(catalog.groups.iter().any(|group| group.id == "recon"));
        assert!(catalog
            .tools
            .iter()
            .any(|tool| tool.id == "nmap" && tool.fedora == "official"));
        assert!(catalog
            .tools
            .iter()
            .any(|tool| tool.id == "sqlmap" && tool.fedora == "upstream"));
    }

    #[test]
    fn duplicate_tool_argument_is_one_row() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/tools.toml");
        let catalog = load(&path).expect("catalog");
        let rows = catalog
            .select(None, &["nmap".to_string(), "nmap".to_string()])
            .expect("select");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "nmap");
    }

    #[test]
    fn rejects_path_separators_in_names() {
        let dir = std::env::temp_dir().join(format!("t2offsec-hostile-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let binary = dir.join("binary.toml");
        fs::write(
            &binary,
            r#"
[meta]
fedora_release = "44"
checked = "2026-10-16"
[[groups]]
id = "recon"
title = "Recon"
summary = "Test."
[[tools]]
id = "bad"
group = "recon"
name = "Bad"
purpose = "Bad."
upstream = "https://example.com"
license = "Apache-2.0"
fedora = "official"
fedora_package = "nmap"
binary = "a/b"
"#,
        )
        .unwrap();
        let err = load(&binary).expect_err("slash binary");
        assert!(err.contains("binary"), "{err}");
        let package = dir.join("package.toml");
        fs::write(
            &package,
            r#"
[meta]
fedora_release = "44"
checked = "2026-10-16"
[[groups]]
id = "recon"
title = "Recon"
summary = "Test."
[[tools]]
id = "bad"
group = "recon"
name = "Bad"
purpose = "Bad."
upstream = "https://example.com"
license = "Apache-2.0"
fedora = "official"
fedora_package = "foo;bar"
binary = "nmap"
"#,
        )
        .unwrap();
        let err = load(&package).expect_err("package");
        assert!(err.contains("fedora_package"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_catalog_skips_parent_directories() {
        let tmp = std::env::temp_dir().join(format!("t2offsec-resolve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let parent = tmp.join("parent");
        let child = parent.join("child");
        fs::create_dir_all(parent.join("catalog")).unwrap();
        fs::create_dir_all(&child).unwrap();
        let parent_catalog = parent.join("catalog").join("tools.toml");
        fs::write(&parent_catalog, "not-used").unwrap();
        let installed = tmp.join("not-installed.toml");
        let cwd_catalog = child.join("catalog").join("tools.toml");
        let err = resolve_catalog(None, None, &installed, Some(&cwd_catalog)).expect_err("parent");
        assert!(err.contains("catalog"), "{err}");
        let found = resolve_catalog(
            Some(parent_catalog.to_str().unwrap()),
            None,
            &installed,
            Some(&cwd_catalog),
        )
        .unwrap();
        assert_eq!(found, parent_catalog);
        fs::create_dir_all(child.join("catalog")).unwrap();
        fs::write(&cwd_catalog, "dev").unwrap();
        let found = resolve_catalog(None, None, &installed, Some(&cwd_catalog)).unwrap();
        assert_eq!(found, cwd_catalog);
        let _ = fs::remove_dir_all(&tmp);
    }
}
