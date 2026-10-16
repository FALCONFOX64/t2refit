# SPDX-License-Identifier: Apache-2.0

"""Load and validate catalog/tools.toml. No package installs and no network."""

from __future__ import annotations

import os
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

FEDORA_KINDS = frozenset({"official", "copr", "upstream"})
ID_OK = set("abcdefghijklmnopqrstuvwxyz0123456789-")


@dataclass(frozen=True)
class Group:
    id: str
    title: str
    summary: str
    kali_metapackages: tuple[str, ...] = ()


@dataclass(frozen=True)
class Tool:
    id: str
    group: str
    name: str
    purpose: str
    upstream: str
    license: str
    fedora: str
    fedora_package: str = ""
    fedora_note: str = ""
    copr: str = ""
    binary: str = ""
    version_args: tuple[str, ...] = ()
    python_distribution: str = ""
    also_in: tuple[str, ...] = ()
    license_note: str = ""


@dataclass(frozen=True)
class Catalog:
    fedora_release: str
    checked: str
    groups: tuple[Group, ...]
    tools: tuple[Tool, ...]
    path: Path
    _by_group: dict[str, Group] = field(default_factory=dict, compare=False)
    _by_tool: dict[str, Tool] = field(default_factory=dict, compare=False)

    def group(self, group_id: str) -> Group:
        try:
            return self._by_group[group_id]
        except KeyError as exc:
            known = ", ".join(g.id for g in self.groups)
            raise KeyError(f"unknown group {group_id!r}; known groups: {known}") from exc

    def tool(self, tool_id: str) -> Tool:
        try:
            return self._by_tool[tool_id]
        except KeyError as exc:
            raise KeyError(f"unknown tool {tool_id!r}") from exc

    def select(self, group_id: str | None = None, tool_ids: list[str] | None = None) -> list[Tool]:
        chosen = list(self.tools)
        if group_id:
            self.group(group_id)
            chosen = [
                tool
                for tool in chosen
                if tool.group == group_id or group_id in tool.also_in
            ]
        if tool_ids:
            wanted = []
            for tool_id in tool_ids:
                wanted.append(self.tool(tool_id))
            chosen = [tool for tool in chosen if tool.id in {item.id for item in wanted}]
            missing = [tool_id for tool_id in tool_ids if tool_id not in {tool.id for tool in chosen}]
            if missing and group_id:
                raise KeyError(
                    f"tool {missing[0]!r} is not in group {group_id!r}"
                )
        return chosen


def find_catalog(explicit: str | None = None) -> Path:
    if explicit:
        path = Path(explicit)
        if not path.is_file():
            raise FileNotFoundError(f"catalog not found: {path}")
        return path
    env = os.environ.get("T2OFFSEC_CATALOG")
    if env:
        path = Path(env)
        if not path.is_file():
            raise FileNotFoundError(f"T2OFFSEC_CATALOG is not a file: {path}")
        return path
    starts = [Path.cwd(), Path(__file__).resolve()]
    seen: set[Path] = set()
    for start in starts:
        for candidate in [start, *start.parents]:
            if candidate in seen:
                continue
            seen.add(candidate)
            path = candidate / "catalog" / "tools.toml"
            if path.is_file():
                return path
    raise FileNotFoundError(
        "catalog/tools.toml not found; run from the t2refit repo or set T2OFFSEC_CATALOG"
    )


def load_catalog(path: Path | None = None) -> Catalog:
    catalog_path = path or find_catalog()
    with catalog_path.open("rb") as handle:
        raw = tomllib.load(handle)
    return _parse(raw, catalog_path)


def _require_str(row: dict, key: str, where: str) -> str:
    value = row.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{where} is missing {key}")
    if "\n" in value or "\r" in value:
        raise ValueError(f"{where} field {key} must be one line")
    return value.strip()


def _block(row: dict, key: str, where: str) -> str:
    value = row.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{where} is missing {key}")
    return value.strip()


def _optional_str(row: dict, key: str, where: str) -> str:
    value = row.get(key, "")
    if value is None:
        return ""
    if not isinstance(value, str):
        raise ValueError(f"{where} field {key} must be a string")
    if "\n" in value or "\r" in value:
        raise ValueError(f"{where} field {key} must be one line")
    return value.strip()


def _id(value: str, where: str) -> str:
    if not value or any(ch not in ID_OK for ch in value):
        raise ValueError(f"{where} id {value!r} must be lowercase letters, digits, and hyphens")
    return value


def _parse(raw: dict, path: Path) -> Catalog:
    meta = raw.get("meta")
    if not isinstance(meta, dict):
        raise ValueError("catalog is missing [meta]")
    fedora_release = _require_str(meta, "fedora_release", "[meta]")
    checked = _require_str(meta, "checked", "[meta]")

    groups: list[Group] = []
    seen_groups: set[str] = set()
    for index, row in enumerate(raw.get("groups") or []):
        if not isinstance(row, dict):
            raise ValueError(f"groups[{index}] is not a table")
        where = f"groups[{index}]"
        group_id = _id(_require_str(row, "id", where), where)
        if group_id in seen_groups:
            raise ValueError(f"duplicate group id {group_id}")
        seen_groups.add(group_id)
        packages = row.get("kali_metapackages") or []
        if not isinstance(packages, list) or not all(isinstance(item, str) for item in packages):
            raise ValueError(f"{where} kali_metapackages must be a list of strings")
        groups.append(
            Group(
                id=group_id,
                title=_require_str(row, "title", where),
                summary=_block(row, "summary", where),
                kali_metapackages=tuple(packages),
            )
        )

    tools: list[Tool] = []
    seen_tools: set[str] = set()
    for index, row in enumerate(raw.get("tools") or []):
        if not isinstance(row, dict):
            raise ValueError(f"tools[{index}] is not a table")
        where = f"tools[{index}]"
        tool_id = _id(_require_str(row, "id", where), where)
        if tool_id in seen_tools:
            raise ValueError(f"duplicate tool id {tool_id}")
        seen_tools.add(tool_id)
        group_id = _require_str(row, "group", where)
        if group_id not in seen_groups:
            raise ValueError(f"{where} group {group_id!r} is not a known group")
        fedora = _require_str(row, "fedora", where)
        if fedora not in FEDORA_KINDS:
            raise ValueError(f"{where} fedora must be official, copr, or upstream")
        package = _optional_str(row, "fedora_package", where)
        copr = _optional_str(row, "copr", where)
        if fedora == "official" and not package:
            raise ValueError(f"{where} official tools need fedora_package")
        if fedora == "copr":
            if not package:
                raise ValueError(f"{where} copr tools need fedora_package")
            owner, sep, name = copr.partition("/")
            if not sep or not owner or not name or "/" in name:
                raise ValueError(f"{where} copr must look like owner/project")
        elif copr:
            raise ValueError(f"{where} copr is only valid when fedora = copr")
        upstream = _require_str(row, "upstream", where)
        if not upstream.startswith(("https://", "http://")):
            raise ValueError(f"{where} upstream must be an http(s) URL")
        version_args = row.get("version_args") or []
        if not isinstance(version_args, list) or not all(isinstance(item, str) for item in version_args):
            raise ValueError(f"{where} version_args must be a list of strings")
        for arg in version_args:
            if not arg.startswith("-") or any(ch.isspace() for ch in arg):
                raise ValueError(f"{where} version_args entries must be dash flags")
        also_in = row.get("also_in") or []
        if not isinstance(also_in, list) or not all(isinstance(item, str) for item in also_in):
            raise ValueError(f"{where} also_in must be a list of strings")
        for extra in also_in:
            if extra not in seen_groups:
                raise ValueError(f"{where} also_in group {extra!r} is not a known group")
            if extra == group_id:
                raise ValueError(f"{where} also_in repeats its own group")
        binary = _optional_str(row, "binary", where)
        distribution = _optional_str(row, "python_distribution", where)
        if not binary and not package and not distribution:
            raise ValueError(f"{where} needs a binary, fedora_package, or python_distribution")
        tools.append(
            Tool(
                id=tool_id,
                group=group_id,
                name=_require_str(row, "name", where),
                purpose=_require_str(row, "purpose", where),
                upstream=upstream,
                license=_require_str(row, "license", where),
                fedora=fedora,
                fedora_package=package,
                fedora_note=_optional_str(row, "fedora_note", where),
                copr=copr,
                binary=binary,
                version_args=tuple(version_args),
                python_distribution=distribution,
                also_in=tuple(also_in),
                license_note=_optional_str(row, "license_note", where),
            )
        )

    by_group = {group.id: group for group in groups}
    by_tool = {tool.id: tool for tool in tools}
    return Catalog(
        fedora_release=fedora_release,
        checked=checked,
        groups=tuple(groups),
        tools=tuple(tools),
        path=path,
        _by_group=by_group,
        _by_tool=by_tool,
    )


def official_packages(catalog: Catalog, group_id: str | None = None) -> list[str]:
    tools = catalog.select(group_id) if group_id else list(catalog.tools)
    names = [tool.fedora_package for tool in tools if tool.fedora == "official" and tool.fedora_package]
    return sorted(set(names))
