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
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]

# The regression corpora that shipped test targets embed with `include_str!`.
# `/tests` is in the archive, so a consumer running `cargo test` on the
# unpacked crate compiles those files; without these entries the compilation
# fails with "couldn't read tests/../docs/...". They are the only files the
# allow-list takes out of an otherwise unpublished directory, and they are
# named one by one rather than by their directories.
EMBEDDED_TEST_FIXTURES = [
    "/docs/reports/2026-09-14-correctness-fixes/raw/cmark-oracle/results.json",
    "/docs/reports/2026-09-14-reference-compatibility/raw/bindings-before-aligned/results.json",
    "/benchmarks/compatibility-audit/fixtures/gfm-examples.json",
]

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
    *EMBEDDED_TEST_FIXTURES,
]

# Directories that exist beside the crate and must never enter the archive as
# directories; only the named fixtures above may come out of one.
NEVER_PUBLISHED = ["docs", "benchmarks", "homepage", "node", "scripts", ".github"]

# The archive's compiled targets. Every file they embed has to be in the
# archive too, which is the rule the fixtures above exist for.
SHIPPED_SOURCE_DIRECTORIES = ["src", "tests", "benches", "examples"]

EMBEDS = re.compile(r'include_(?:str|bytes)!\(\s*"([^"]+)"')


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
            if package["package"]["name"] == "ferromark-transforms":
                self.assertEqual(package["package"]["publish"], ["crates-io"], member)
            else:
                self.assertIs(package["package"]["publish"], False, member)
            requirement = package.get("dependencies", {}).get("ferromark")
            if requirement is not None:
                # A path dependency without an explicit version is skipped by
                # Release Please and would silently keep the previous release.
                if package["package"]["name"] == "ferromark-transforms":
                    expected_requirement = f"={version}"
                else:
                    expected_requirement = version
                self.assertEqual(requirement["version"], expected_requirement, member)
                expected_path = "/".join([".."] * len(Path(member).parts))
                self.assertEqual(requirement["path"], expected_path, member)

    def test_transforms_is_the_only_published_workspace_member(self):
        published = []
        for member in self.manifest["workspace"]["members"]:
            package = tomllib.loads((ROOT / member / "Cargo.toml").read_text())["package"]
            if package.get("publish") is not False:
                published.append(package["name"])
        self.assertEqual(published, ["ferromark-transforms"])

    def test_transforms_depends_on_an_exact_core_version(self):
        manifest = tomllib.loads((ROOT / "transforms" / "Cargo.toml").read_text())
        dependency = manifest["dependencies"]["ferromark"]
        self.assertEqual(dependency["version"], f"={self.manifest['package']['version']}")
        self.assertEqual(dependency["path"], "..")

    def test_core_does_not_depend_on_the_optional_transform_crate(self):
        self.assertNotIn("ferromark-transforms", self.manifest["dependencies"])

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
        entries = [entry for entry in self.include if entry not in EMBEDDED_TEST_FIXTURES]
        included = {entry.lstrip("/").split("/", 1)[0] for entry in entries}
        for directory in NEVER_PUBLISHED:
            self.assertTrue((ROOT / directory).is_dir(), f"{directory}: expected in the repository")
            self.assertNotIn(directory, included)

    def test_embedded_fixtures_are_individual_files(self):
        for fixture in EMBEDDED_TEST_FIXTURES:
            self.assertIn(fixture, self.include, fixture)
            self.assertTrue((ROOT / fixture.lstrip("/")).is_file(), f"{fixture}: not a file")

    def test_every_file_a_shipped_target_embeds_is_in_the_archive(self):
        # The guard the published crate needs: `cargo test` on the unpacked
        # archive compiles `tests/`, `benches/` and `examples/`, so an
        # `include_str!` that reaches outside the allow-list breaks it there
        # and nowhere else.
        included = [(ROOT / entry.lstrip("/")).resolve() for entry in self.include]
        for directory in SHIPPED_SOURCE_DIRECTORIES:
            for source in sorted((ROOT / directory).rglob("*.rs")):
                for embedded in EMBEDS.findall(source.read_text()):
                    where = f"{source.relative_to(ROOT)}: {embedded}"
                    target = (source.parent / embedded).resolve()
                    self.assertTrue(target.is_file(), f"{where}: missing")
                    self.assertTrue(
                        any(target == entry or entry in target.parents for entry in included),
                        f"{where}: outside the published archive",
                    )

    def test_upstream_attribution_ships_with_the_crate(self):
        for attribution in ("/LICENSE", "/LICENSE-MIT", "/UPSTREAM.md"):
            self.assertIn(attribution, self.include)


class TransformArchive(unittest.TestCase):
    def setUp(self):
        self.manifest = tomllib.loads((ROOT / "transforms" / "Cargo.toml").read_text())

    def test_transform_crate_has_a_narrow_archive_allow_list(self):
        expected = [
            "/src",
            "/tests",
            "/examples",
            "/data",
            "/README.md",
        ]
        self.assertEqual(self.manifest["package"]["include"], expected)
        for entry in expected:
            self.assertTrue((ROOT / "transforms" / entry.lstrip("/")).exists(), entry)

    def test_transform_package_inherits_the_workspace_license(self):
        self.assertTrue(self.manifest["package"]["license"]["workspace"])
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]
        self.assertEqual(workspace["license"], "MIT")


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
