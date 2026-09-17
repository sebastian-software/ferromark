#!/usr/bin/env python3
"""Contract for the published Cargo archive and the release template.

`ferromark` is the repository root package, so Cargo would otherwise offer the
whole repository to crates.io. The `include` allow-list is what keeps the
archive at the file set the nested crate shipped, and this test is the guard
the blueprint asks for ("moving a package to the repository root can broaden
the files Cargo includes").
"""
import json
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]

# Everything the crate ships, anchored to the package root. `Cargo.toml`,
# `Cargo.lock` and the VCS info file are added by Cargo itself.
EXPECTED_INCLUDE = [
    "/src",
    "/tests",
    "/benches",
    "/examples",
    "/README.md",
    "/LICENSE",
    "/LICENSE-MIT",
    "/UPSTREAM.md",
]

# Directories that exist beside the crate and must never enter the archive.
NEVER_PUBLISHED = ["docs", "benchmarks", "homepage", "node", "scripts", ".github"]


class RootPackage(unittest.TestCase):
    def setUp(self):
        self.manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())

    def test_root_package_carries_a_concrete_version(self):
        package = self.manifest["package"]
        self.assertEqual(package["name"], "ferromark")
        self.assertIsInstance(package["version"], str)
        # Release Please cannot replace an inherited workspace version.
        self.assertNotIn("version", self.manifest["workspace"].get("package", {}))

    def test_members_carry_concrete_versions_and_requirements(self):
        version = self.manifest["package"]["version"]
        for member in self.manifest["workspace"]["members"]:
            package = tomllib.loads((ROOT / member / "Cargo.toml").read_text())
            self.assertEqual(package["package"]["version"], version, member)
            self.assertIs(package["package"]["publish"], False, member)
            requirement = package.get("dependencies", {}).get("ferromark")
            if requirement is not None:
                # A path dependency without an explicit version is skipped by
                # Release Please and would silently keep the previous release.
                self.assertEqual(requirement["version"], version, member)
                self.assertEqual(requirement["path"], "../..", member)

    def test_no_simple_strategy_leftovers(self):
        self.assertFalse((ROOT / "version.txt").exists())
        self.assertNotIn("x-release-please", (ROOT / "Cargo.toml").read_text())


class PublishedArchive(unittest.TestCase):
    def setUp(self):
        self.include = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["include"]

    def test_include_is_the_reviewed_allow_list(self):
        self.assertEqual(self.include, EXPECTED_INCLUDE)

    def test_every_included_entry_is_anchored_and_exists(self):
        for entry in self.include:
            self.assertTrue(entry.startswith("/"), f"{entry}: must be anchored to the package root")
            self.assertTrue((ROOT / entry.lstrip("/")).exists(), f"{entry}: missing")

    def test_the_repository_around_the_crate_stays_out(self):
        included = {entry.lstrip("/").split("/", 1)[0] for entry in self.include}
        for directory in NEVER_PUBLISHED:
            self.assertTrue((ROOT / directory).is_dir(), f"{directory}: expected in the repository")
            self.assertNotIn(directory, included)

    def test_upstream_attribution_ships_with_the_crate(self):
        for attribution in ("/LICENSE", "/LICENSE-MIT", "/UPSTREAM.md"):
            self.assertIn(attribution, self.include)


class ReleaseTemplate(unittest.TestCase):
    def setUp(self):
        self.config = json.loads((ROOT / "release-please-config.json").read_text())

    def test_the_rust_strategy_owns_every_cargo_file(self):
        self.assertEqual(self.config["release-type"], "rust")
        for entry in self.config["packages"]["."]["extra-files"]:
            path = entry if isinstance(entry, str) else entry["path"]
            self.assertFalse(path.endswith(("Cargo.toml", "Cargo.lock")), path)
            self.assertFalse(path.endswith("lock.yaml"), path)

    def test_the_tag_keeps_its_published_shape(self):
        # `v2.0.0-rc.1` is published; the component must not enter the tag.
        self.assertIs(self.config["include-component-in-tag"], False)
        self.assertEqual(self.config["packages"]["."]["component"], "ferromark")


if __name__ == "__main__":
    unittest.main()
