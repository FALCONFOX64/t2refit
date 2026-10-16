# SPDX-License-Identifier: Apache-2.0

"""Read-only presence checks. This module never executes catalog tools."""

from __future__ import annotations

import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from t2offsec.catalog import Tool

_DIST_NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.-]{0,80}$")
_EXTRA_DIRS = ("/usr/bin", "/usr/sbin", "/bin", "/sbin")


@dataclass(frozen=True)
class Presence:
    tool_id: str
    group: str
    name: str
    present: bool
    path: str
    rpm_version: str
    python_version: str
    via: str


def find_binary(name: str, path_env: str | None = None) -> str:
    if not name:
        return ""
    raw = os.environ.get("PATH", "") if path_env is None else path_env
    directories = [item for item in raw.split(os.pathsep) if item]
    for extra in _EXTRA_DIRS:
        if extra not in directories:
            directories.append(extra)
    for directory in directories:
        candidate = Path(directory) / name
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    return ""


def rpm_versions(packages: list[str]) -> dict[str, str] | None:
    """Return package to version, or None when rpm is not installed.

    Missing packages are omitted. The query is read-only.
    """
    names = sorted({package for package in packages if package})
    if not names:
        return {}
    if shutil.which("rpm") is None:
        return None
    completed = subprocess.run(
        ["rpm", "-q", "--qf", "%{NAME}\\t%{VERSION}-%{RELEASE}\\n", *names],
        check=False,
        capture_output=True,
        text=True,
    )
    found: dict[str, str] = {}
    for line in completed.stdout.splitlines():
        package, sep, version = line.partition("\t")
        if sep and package and version and "not installed" not in line:
            found[package] = version.strip()
    return found


def python_distribution_version(distribution: str) -> str:
    if not distribution or _DIST_NAME.fullmatch(distribution) is None:
        return ""
    import importlib.metadata as metadata

    try:
        return metadata.version(distribution)
    except metadata.PackageNotFoundError:
        return ""


def check_tools(tools: list[Tool], path_env: str | None = None) -> list[Presence]:
    rpm = rpm_versions([tool.fedora_package for tool in tools if tool.fedora_package])
    rows: list[Presence] = []
    for tool in tools:
        path = find_binary(tool.binary, path_env)
        rpm_version = ""
        if tool.fedora_package and rpm is not None:
            rpm_version = rpm.get(tool.fedora_package, "")
        py_version = python_distribution_version(tool.python_distribution)
        via: list[str] = []
        if path:
            via.append("path")
        if rpm_version:
            via.append("rpm")
        if py_version:
            via.append("python")
        rows.append(
            Presence(
                tool_id=tool.id,
                group=tool.group,
                name=tool.name,
                present=bool(via),
                path=path,
                rpm_version=rpm_version,
                python_version=py_version,
                via=",".join(via),
            )
        )
    return rows
