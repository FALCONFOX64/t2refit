// SPDX-License-Identifier: Apache-2.0

//! Fast read-only inventory of the offensive-lab catalog.
//! It does not install packages and it does not contact the network.

mod catalog;
mod inventory;

use catalog::{find_catalog, load};
use inventory::{inventory, SystemLookup};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("t2offsec-inventory: {err}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<u8, String> {
    let mut catalog_path: Option<String> = None;
    let mut group: Option<String> = None;
    let mut tools: Vec<String> = Vec::new();
    let mut as_json = false;
    let mut strict = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => as_json = true,
            "--strict" => strict = true,
            "--help" | "-h" => {
                print_help();
                return Ok(0);
            }
            "--version" => {
                println!("t2offsec-inventory {}", env!("CARGO_PKG_VERSION"));
                return Ok(0);
            }
            "--catalog" => {
                catalog_path = Some(iter.next().ok_or("--catalog needs a path")?);
            }
            "--group" => {
                group = Some(iter.next().ok_or("--group needs an id")?);
            }
            "--tool" => {
                tools.push(iter.next().ok_or("--tool needs an id")?);
            }
            other if other.starts_with('-') => return Err(format!("unknown flag {other}")),
            other => return Err(format!("unexpected argument {other}")),
        }
    }

    let path = find_catalog(catalog_path.as_deref())?;
    let loaded = load(&path)?;
    let selected = loaded.select(group.as_deref(), &tools)?;
    let rows = inventory(&selected, &SystemLookup);
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows_json(&rows)).map_err(|err| err.to_string())?
        );
    } else {
        for row in &rows {
            let state = if row.present { "present" } else { "absent" };
            if row.tool_version.is_empty() && row.rpm_version.is_empty() && row.path.is_empty() {
                println!("{}\t{}\t{}", row.group, row.id, state);
            } else {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    row.group,
                    row.id,
                    state,
                    dash(&row.path),
                    dash(&row.rpm_version),
                    dash(&row.tool_version)
                );
            }
        }
    }
    if strict && rows.iter().any(|row| !row.present) {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn dash(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}

fn rows_json(rows: &[inventory::Row]) -> Vec<serde_json::Value> {
    rows.iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "group": row.group,
                "present": row.present,
                "path": row.path,
                "rpm_version": row.rpm_version,
                "tool_version": row.tool_version,
                "note": row.note,
            })
        })
        .collect()
}

fn print_help() {
    println!(
        "\
t2offsec-inventory — read-only install and version inventory

Usage:
  t2offsec-inventory [--catalog PATH] [--group ID] [--tool ID] [--json] [--strict]

The command looks up binaries and rpm packages. It runs a version flag only
when the catalog lists one and the binary is not on the deny list. It does
not install packages and it does not open network connections of its own.
Absent tools are a normal result. --strict exits 1 when any selected tool
is absent."
    );
}
