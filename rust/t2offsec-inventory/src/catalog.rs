// SPDX-License-Identifier: Apache-2.0

//! Load catalog/tools.toml. Parsing does not touch the network.
//!
//! Several fields are rendered by the Python CLI and the Markdown catalog.
//! This binary only needs ids, packages, and version flags.
#![allow(dead_code)]

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

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
                Some(group_id) => tool.group == group_id || tool.also_in.iter().any(|item| item == group_id),
            })
            .cloned()
            .collect();
        if !tools.is_empty() {
            for tool_id in tools {
                if !self.tools.iter().any(|tool| tool.id == *tool_id) {
                    return Err(format!("unknown tool {tool_id}"));
                }
            }
            chosen.retain(|tool| tools.iter().any(|tool_id| tool_id == &tool.id));
            if chosen.len() != tools.len() {
                return Err("one of the tools is not in that group".to_string());
            }
        }
        Ok(chosen)
    }
}

pub fn load(path: &Path) -> Result<Catalog, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    toml::from_str(&text).map_err(|err| format!("parse {}: {err}", path.display()))
}

pub fn find_catalog(explicit: Option<&str>) -> Result<PathBuf, String> {
    if let Some(value) = explicit {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("catalog not found: {}", path.display()));
    }
    if let Ok(value) = std::env::var("T2OFFSEC_CATALOG") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("T2OFFSEC_CATALOG is not a file: {}", path.display()));
    }
    let mut starts = vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))];
    if let Ok(exe) = std::env::current_exe() {
        starts.push(exe);
    }
    for start in starts {
        for candidate in start.ancestors() {
            let path = candidate.join("catalog").join("tools.toml");
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    Err("catalog/tools.toml not found; run from the t2refit repo or set T2OFFSEC_CATALOG".to_string())
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
        assert!(catalog.tools.iter().any(|tool| tool.id == "nmap" && tool.fedora == "official"));
        assert!(catalog.tools.iter().any(|tool| tool.id == "sqlmap" && tool.fedora == "upstream"));
    }
}
