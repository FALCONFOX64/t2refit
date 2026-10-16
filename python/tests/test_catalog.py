# SPDX-License-Identifier: Apache-2.0

"""Catalog schema, presence checks, and the generated Markdown page."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from t2offsec.catalog import find_catalog, load_catalog
from t2offsec.cli import main
from t2offsec.presence import check_tools, find_binary
from t2offsec.render import render_markdown

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
