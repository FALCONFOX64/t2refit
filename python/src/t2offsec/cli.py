# SPDX-License-Identifier: Apache-2.0

"""Command line for the offensive catalog. Presence checks are read-only."""

from __future__ import annotations

import argparse
import json
import os
import sys

from t2offsec import __version__
from t2offsec.catalog import find_catalog, load_catalog
from t2offsec.presence import check_tools
from t2offsec.render import render_manual, render_markdown


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="t2offsec",
        description=(
            "List the Fedora offensive-lab catalog and check whether selected "
            "tools are installed. This command does not install packages and "
            "does not run the tools."
        ),
    )
    parser.add_argument("--catalog", help="path to catalog/tools.toml")
    parser.add_argument("--version", action="version", version=f"t2offsec {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("groups", help="list catalog groups")

    list_parser = sub.add_parser("list", help="list tools")
    list_parser.add_argument("--group", help="limit to one group, including cross-references")

    check_parser = sub.add_parser("check", help="check whether tools are present")
    check_parser.add_argument("--group", help="limit to one group, including cross-references")
    check_parser.add_argument("--tool", action="append", default=[], help="tool id; repeatable")
    check_parser.add_argument("--json", action="store_true")
    check_parser.add_argument(
        "--strict",
        action="store_true",
        help="exit 1 when any selected tool is absent",
    )

    pkg_parser = sub.add_parser(
        "packages",
        help="print official Fedora package names for a group's own tools",
    )
    pkg_parser.add_argument("--group", help="limit to one primary group")

    sub.add_parser("gaps", help="list tools with no official Fedora package")
    sub.add_parser("catalog", help="print the Markdown catalog")
    sub.add_parser("manual", help="print the raccoonix-t2 user manual")
    return parser


def _silence_broken_pipe() -> None:
    """Keep a closed pipe from becoming Python 3.14's exit status 120.

    The interpreter flushes sys.stdout again during shutdown. If that
    flush still sees the closed pipe, the ignored BrokenPipeError
    replaces the status returned here. A new stream, plus fd 1 pointed
    at /dev/null, gives that flush somewhere to succeed.
    """
    try:
        sys.stdout = open(os.devnull, "w")
    except OSError:
        pass
    try:
        null_fd = os.open(os.devnull, os.O_WRONLY)
    except OSError:
        return
    try:
        os.dup2(null_fd, 1)
    finally:
        os.close(null_fd)


def main(argv: list[str] | None = None) -> int:
    try:
        code = _run(argv)
        # A large write can sit in the block buffer until shutdown.
        # Flush here so a closed pipe raises inside this handler.
        sys.stdout.flush()
    except BrokenPipeError:
        _silence_broken_pipe()
        return 0
    return code


def _run(argv: list[str] | None) -> int:
    parser = _parser()
    args = parser.parse_args(argv)
    try:
        catalog = load_catalog(find_catalog(args.catalog))
    except (OSError, ValueError, KeyError) as exc:
        print(f"t2offsec: {exc}", file=sys.stderr)
        return 2

    try:
        if args.command == "groups":
            return _groups(catalog)
        if args.command == "list":
            return _list(catalog, args.group)
        if args.command == "check":
            return _check(catalog, args.group, args.tool, args.json, args.strict)
        if args.command == "packages":
            return _packages(catalog, args.group)
        if args.command == "gaps":
            return _gaps(catalog)
        if args.command == "catalog":
            sys.stdout.write(render_markdown(catalog))
            return 0
        if args.command == "manual":
            sys.stdout.write(render_manual(catalog))
            return 0
    except KeyError as exc:
        print(f"t2offsec: {exc}", file=sys.stderr)
        return 2
    print("t2offsec: unknown command", file=sys.stderr)
    return 2


def _groups(catalog) -> int:
    for group in catalog.groups:
        kali = ",".join(group.kali_metapackages)
        print(f"{group.id}\t{group.title}\t{kali}")
    return 0


def _list(catalog, group_id: str | None) -> int:
    tools = catalog.select(group_id)
    for tool in tools:
        where = tool.fedora if tool.fedora != "copr" else f"copr:{tool.copr}"
        print(f"{tool.group}\t{tool.id}\t{tool.name}\t{where}\t{tool.purpose}")
    return 0


def _check(catalog, group_id: str | None, tool_ids: list[str], as_json: bool, strict: bool) -> int:
    tools = catalog.select(group_id, tool_ids or None)
    rows = check_tools(tools)
    if as_json:
        json.dump([row.__dict__ for row in rows], sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        for row in rows:
            state = "present" if row.present else "absent"
            detail = row.via or "-"
            print(f"{row.group}\t{row.tool_id}\t{state}\t{detail}")
    if strict and any(not row.present for row in rows):
        return 1
    return 0


def _packages(catalog, group_id: str | None) -> int:
    if group_id:
        catalog.group(group_id)
        tools = [tool for tool in catalog.tools if tool.group == group_id]
    else:
        tools = list(catalog.tools)
    names = sorted({tool.fedora_package for tool in tools if tool.fedora == "official" and tool.fedora_package})
    for name in names:
        print(name)
    return 0


def _gaps(catalog) -> int:
    for tool in catalog.tools:
        if tool.fedora == "official":
            continue
        if tool.fedora == "copr":
            path = f"copr {tool.copr} {tool.fedora_package}"
        else:
            path = "upstream"
        print(f"{tool.group}\t{tool.id}\t{path}\t{tool.upstream}")
    return 0
