# SPDX-License-Identifier: Apache-2.0

"""Catalog schema, presence checks, and the generated Markdown page."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from t2offsec import __version__
from t2offsec.catalog import (
    INSTALLED_CATALOG,
    PACKAGE_NAME,
    PACKAGE_VERSION,
    find_catalog,
    load_catalog,
    valid_command_name,
)
from t2offsec.cli import main
from t2offsec.presence import check_tools, find_binary, rpm_argv
from t2offsec.render import help_command, render_manual, render_markdown

ROOT = Path(__file__).resolve().parents[2]
KALI_TOOLS = {
    "kali-tools-802-11",
    "kali-tools-bluetooth",
    "kali-tools-crypto-stego",
    "kali-tools-database",
    "kali-tools-detect",
    "kali-tools-exploitation",
    "kali-tools-forensics",
    "kali-tools-fuzzing",
    "kali-tools-gpu",
    "kali-tools-hardware",
    "kali-tools-identify",
    "kali-tools-information-gathering",
    "kali-tools-passwords",
    "kali-tools-post-exploitation",
    "kali-tools-protect",
    "kali-tools-recover",
    "kali-tools-reporting",
    "kali-tools-respond",
    "kali-tools-reverse-engineering",
    "kali-tools-rfid",
    "kali-tools-sdr",
    "kali-tools-sniffing-spoofing",
    "kali-tools-social-engineering",
    "kali-tools-top10",
    "kali-tools-voip",
    "kali-tools-vulnerability",
    "kali-tools-web",
    "kali-tools-windows-resources",
    "kali-tools-wireless",
}
FORBIDDEN = (
    "msfvenom",
    "meterpreter",
    "reverse shell",
    "rockyou",
    "powershell -enc",
    "hashcat -m",
    "aircrack-ng -w",
)


class CatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = load_catalog(find_catalog(str(ROOT / "catalog" / "tools.toml")))

    def test_kali_menus_are_covered(self):
        found = {name for group in self.catalog.groups for name in group.kali_metapackages}
        self.assertEqual(KALI_TOOLS, found & KALI_TOOLS)
        self.assertTrue(KALI_TOOLS <= found)

    def test_ids_and_fedora_kinds(self):
        self.assertGreater(len(self.catalog.tools), 80)
        official = [tool for tool in self.catalog.tools if tool.fedora == "official"]
        self.assertGreater(len(official), 40)
        self.assertIn("nmap", {tool.id for tool in self.catalog.tools})
        self.assertEqual(self.catalog.tool("nmap").fedora_package, "nmap")
        self.assertEqual(self.catalog.tool("sqlmap").fedora, "upstream")
        self.assertEqual(self.catalog.tool("ghidra").copr, "davidgf/ghidra")

    def test_top10_cross_reference(self):
        top = self.catalog.select("top10")
        ids = {tool.id for tool in top}
        for required in ("nmap", "aircrack-ng", "hydra", "john", "metasploit", "sqlmap", "tshark", "responder"):
            self.assertIn(required, ids)

    def test_generated_markdown_matches(self):
        rendered = render_markdown(self.catalog)
        path = ROOT / "docs" / "offensive" / "catalog.md"
        self.assertTrue(path.is_file(), "docs/offensive/catalog.md is missing")
        self.assertEqual(path.read_text(encoding="utf-8"), rendered)

    def test_generated_manual_matches(self):
        rendered = render_manual(self.catalog)
        path = ROOT / "docs" / "offensive" / "manual.md"
        self.assertTrue(path.is_file(), "docs/offensive/manual.md is missing")
        self.assertEqual(path.read_text(encoding="utf-8"), rendered)
        self.assertIn(PACKAGE_NAME, rendered)
        self.assertIn("nmap --help", rendered)
        self.assertIn("pydoc impacket", rendered)
        for tool in self.catalog.tools:
            self.assertIn(tool.name, rendered)
            self.assertIn(tool.license, rendered)
            self.assertIn(tool.upstream, rendered)
            command = help_command(tool)
            if command:
                self.assertIn(command, rendered)
        notice = (ROOT / "NOTICE").read_text(encoding="utf-8")
        release = (ROOT / "docs" / "offensive" / "release-notes.md").read_text(encoding="utf-8")
        spec = (ROOT / "packaging" / f"{PACKAGE_NAME}.spec").read_text(encoding="utf-8")
        for text in (rendered, notice, release, spec):
            lowered = text.lower()
            for banned in ("terminator", "skynet", "cyberdyne"):
                self.assertNotIn(banned, lowered)
            self.assertIn(PACKAGE_NAME, text)
        self.assertIn(str(INSTALLED_CATALOG), spec)
        self.assertEqual(
            str(INSTALLED_CATALOG),
            f"/usr/share/{PACKAGE_NAME}/catalog/tools.toml",
        )

    def test_no_procedure_snippets(self):
        blobs = [(ROOT / "catalog" / "tools.toml").read_text(encoding="utf-8").lower()]
        for path in (ROOT / "docs" / "offensive").glob("*.md"):
            blobs.append(path.read_text(encoding="utf-8").lower())
        text = "\n".join(blobs)
        for needle in FORBIDDEN:
            self.assertNotIn(needle, text)

    def test_check_sees_a_path_binary(self):
        with tempfile.TemporaryDirectory() as tmp:
            binary = Path(tmp) / "nmap"
            binary.write_text("#!/bin/sh\n", encoding="utf-8")
            binary.chmod(0o755)
            rows = check_tools([self.catalog.tool("nmap")], path_env=tmp)
        self.assertEqual(len(rows), 1)
        self.assertTrue(rows[0].present)
        self.assertTrue(rows[0].path.endswith("/nmap"))
        self.assertIn("path", rows[0].via)

    def test_missing_binary_is_absent_without_rpm(self):
        with tempfile.TemporaryDirectory() as tmp:
            rows = check_tools([self.catalog.tool("feroxbuster")], path_env=tmp)
        self.assertFalse(rows[0].present)

    def test_cli_groups_and_unknown_group(self):
        self.assertEqual(main(["--catalog", str(self.catalog.path), "groups"]), 0)
        self.assertEqual(main(["--catalog", str(self.catalog.path), "list", "--group", "nope"]), 2)

    def test_cli_strict_on_empty_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            env_path = os.environ.get("PATH")
            os.environ["PATH"] = tmp
            try:
                code = main(
                    ["--catalog", str(self.catalog.path), "check", "--tool", "feroxbuster", "--strict"]
                )
            finally:
                if env_path is None:
                    os.environ.pop("PATH", None)
                else:
                    os.environ["PATH"] = env_path
        self.assertEqual(code, 1)

    def test_packages_are_official_names_only(self):
        code = main(["--catalog", str(self.catalog.path), "packages", "--group", "lab"])
        self.assertEqual(code, 0)

    def test_find_binary_ignores_directories(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "nmap").mkdir()
            self.assertEqual(find_binary("nmap", tmp), "")

    def test_hostile_names_are_rejected(self):
        self.assertFalse(valid_command_name("../nmap"))
        self.assertFalse(valid_command_name("a/b"))
        self.assertFalse(valid_command_name("a\\b"))
        self.assertFalse(valid_command_name("-flag"))
        self.assertFalse(valid_command_name("foo;rm"))
        self.assertFalse(valid_command_name(""))
        self.assertEqual(find_binary("../sh", "/bin"), "")
        self.assertEqual(find_binary("a/b", "/usr/bin"), "")
        argv = rpm_argv(["nmap", "-evil", "foo/bar", "python3-impacket", "nmap"])
        self.assertEqual(
            argv,
            [
                "rpm",
                "-q",
                "--qf",
                "%{NAME}\\t%{VERSION}-%{RELEASE}\\n",
                "--",
                "nmap",
                "python3-impacket",
            ],
        )
        self.assertEqual(rpm_argv(["../rpm"]), [])

    def test_hostile_catalog_values(self):
        base = """
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
binary = "BIN"
"""
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "tools.toml"
            path.write_text(base.replace("BIN", "../nmap"), encoding="utf-8")
            with self.assertRaises(ValueError):
                load_catalog(path)
            path.write_text(base.replace("BIN", "nmap").replace('fedora_package = "nmap"', 'fedora_package = "foo;bar"'), encoding="utf-8")
            with self.assertRaises(ValueError):
                load_catalog(path)

    def test_catalog_lookup_does_not_walk_parents(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            parent = root / "parent"
            child = parent / "child"
            (parent / "catalog").mkdir(parents=True)
            child.mkdir()
            (parent / "catalog" / "tools.toml").write_text("not-a-catalog\n", encoding="utf-8")
            previous = os.getcwd()
            env = os.environ.pop("T2OFFSEC_CATALOG", None)
            try:
                os.chdir(child)
                if INSTALLED_CATALOG.is_file():
                    self.assertEqual(find_catalog(), INSTALLED_CATALOG)
                else:
                    with self.assertRaises(FileNotFoundError):
                        find_catalog()
            finally:
                os.chdir(previous)
                if env is None:
                    os.environ.pop("T2OFFSEC_CATALOG", None)
                else:
                    os.environ["T2OFFSEC_CATALOG"] = env

    def test_unreadable_or_invalid_catalog_is_a_clean_error(self):
        self.assertEqual(main(["--catalog", "/dev/null", "groups"]), 2)
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(main(["--catalog", tmp, "groups"]), 2)

    def test_broken_pipe_exits_cleanly(self):
        env = {**os.environ, "PYTHONPATH": str(ROOT / "python" / "src")}
        # Close the read end before the child starts so the first flush
        # sees EPIPE. Closing after Popen races the child and can hide
        # the Python 3.14 shutdown-flush failure.
        read_fd, write_fd = os.pipe()
        os.close(read_fd)
        proc = subprocess.Popen(
            [sys.executable, "-m", "t2offsec", "manual"],
            cwd=ROOT,
            env=env,
            stdout=write_fd,
            stderr=subprocess.PIPE,
        )
        os.close(write_fd)
        assert proc.stderr is not None
        err = proc.stderr.read()
        proc.stderr.close()
        code = proc.wait(timeout=20)
        self.assertEqual(code, 0, err.decode(errors="replace"))

    def test_versions_match(self):
        self.assertEqual(__version__, PACKAGE_VERSION)
        self.assertEqual(PACKAGE_VERSION, "0.1.0")
        pyproject = (ROOT / "python" / "pyproject.toml").read_text(encoding="utf-8")
        cargo = (ROOT / "rust" / "t2offsec-inventory" / "Cargo.toml").read_text(encoding="utf-8")
        spec = (ROOT / "packaging" / f"{PACKAGE_NAME}.spec").read_text(encoding="utf-8")
        notes = (ROOT / "docs" / "offensive" / "release-notes.md").read_text(encoding="utf-8")
        self.assertIn(f'version = "{PACKAGE_VERSION}"', pyproject)
        self.assertIn(f'version = "{PACKAGE_VERSION}"', cargo)
        self.assertIn(f"Version:        {PACKAGE_VERSION}", spec)
        self.assertIn(f"# {PACKAGE_NAME} {PACKAGE_VERSION}", notes)
        self.assertIn("t2refit-0.1.0", notes)
        self.assertNotIn("sudo dnf install raccoonix-t2", (ROOT / "docs" / "offensive" / "manual.md").read_text(encoding="utf-8"))


class CliOutputTests(unittest.TestCase):
    def test_gaps_include_sqlmap(self):
        completed = subprocess.run(
            ["python3", "-m", "t2offsec", "gaps"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env={**os.environ, "PYTHONPATH": str(ROOT / "python" / "src")},
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("sqlmap", completed.stdout)
        self.assertIn("metasploit", completed.stdout)
        self.assertNotIn("nmap\tupstream", completed.stdout)


if __name__ == "__main__":
    unittest.main()
