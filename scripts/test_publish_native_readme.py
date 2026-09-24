#!/usr/bin/env python3
"""Contract for the publisher that turns archived native comparisons into published figures.

The README, the homepage and the benchmark guide state one set of figures per
platform, and CI's `--check` only proves they match what this publisher derives.
These tests pin the derivation itself: it must reproduce the figures an archived
report published, name each platform's machine from the report, refuse a report
filed under the wrong platform, and never state "faster" where the archive does
not show it.
"""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "publish-native-readme.py"
# Archived reports are frozen, so this one pins the derivation for good.
ROUND_4 = Path("docs/reports/2026-09-21-native-round-4")


def load_publisher():
    # The script's file name is not an importable module name.
    spec = importlib.util.spec_from_file_location("publish_native_readme", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


publisher = load_publisher()


def linux_shaped_copy(parent):
    """Round 4's evidence, relabelled the way archive.py records a GitHub-hosted Linux run."""
    report = parent / "2026-09-24-native-linux-x86-64"
    shutil.copytree(ROOT / ROUND_4, report)
    run = json.loads((report / "run.json").read_text())
    run["host_before"].update(cpu="AMD EPYC 7763 64-Core Processor",
                              platform="Linux-6.17.0-1022-azure-x86_64-with-glibc2.39",
                              time_utc="2026-09-24T13:22:35Z")
    (report / "run.json").write_text(json.dumps(run))
    build = json.loads((report / "build.json").read_text())
    build["platform"] = {"system": "Linux", "machine": "x86_64"}
    (report / "build.json").write_text(json.dumps(build))
    (report / "checks/commands.json").write_text(json.dumps({
        "workflow_origin": "The [native comparison workflow](https://example.invalid) ran on a "
                           "GitHub-hosted ubuntu-latest runner for a workflow_dispatch event."}))
    for name in ("FLAGS.md", "OUTPUT-REVIEW.md"):
        (report / name).unlink()
    return report


class ArchivedFigures(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.apple = publisher.measure("macos-arm64", ROUND_4)
        cls.published = json.loads(publisher.website_json([cls.apple]))["platforms"][0]

    def test_it_reproduces_the_figures_the_archived_publisher_stated(self):
        fresh = {row["id"]: row["fresh"] for row in self.published["figures"]}
        self.assertEqual(fresh, {"v1": 2.07, "pulldown-cmark": 2.65, "md4c": 3.51, "bun": 6.03,
                                 "ox-content": 1.19})
        self.assertEqual(self.published["documents"],
                         {"all": 57, "fiveEngineAgreement": 50, "sixEngineAgreement": 14, "heldOut": 28})

    def test_it_reproduces_the_held_out_pgo_standing(self):
        pgo = self.published["pgo"]
        self.assertEqual(pgo["v2Gain"], {"fresh": 1.27, "reuse": 1.26})
        self.assertEqual({row["id"]: row["fresh"] for row in pgo["figures"]},
                         {"v1": 2.13, "pulldown-cmark": 2.59, "md4c": 4.24, "bun": 5.75, "ox-content": 1.36})

    def test_it_names_the_machine_and_revision_from_the_report(self):
        self.assertEqual(self.published["machine"], "Apple M1 Pro")
        self.assertFalse(self.published["sharedRunner"])
        self.assertEqual((self.published["revision"], self.published["measured"]), ("39b1f0b7", "2026-09-21"))


class PlatformNames(unittest.TestCase):
    def test_reports_from_before_linux_support_derive_the_platform_from_run_json(self):
        slug = publisher.platform_slug({}, {"host_before": {"platform": "macOS-27.0-arm64-arm-64bit-Mach-O"}})
        self.assertEqual(slug, "macos-arm64")
        slug = publisher.platform_slug({}, {"host_before": {
            "platform": "Linux-6.17.0-1022-azure-x86_64-with-glibc2.39"}})
        self.assertEqual(slug, "linux-x86-64")

    def test_archive_py_reports_name_their_platform_in_build_json(self):
        self.assertEqual(publisher.platform_slug({"platform": {"system": "Darwin", "machine": "arm64"}}, {}),
                         "macos-arm64")

    def test_cpu_models_lose_marketing_suffixes(self):
        self.assertEqual(publisher.cpu_model("AMD EPYC 7763 64-Core Processor"), "AMD EPYC 7763")
        self.assertEqual(publisher.cpu_model("Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz"),
                         "Intel Xeon Platinum 8370C")
        self.assertEqual(publisher.cpu_model("Apple M1 Pro"), "Apple M1 Pro")

    def test_a_bare_architecture_falls_back_to_the_named_machine(self):
        self.assertEqual(publisher.machine_name("arm64", "Apple M1 Pro", False), "Apple M1 Pro")
        with self.assertRaises(SystemExit):
            publisher.machine_name("x86_64", None, True)

    def test_a_github_hosted_run_says_so(self):
        self.assertEqual(publisher.machine_name("AMD EPYC 9V74 80-Core Processor", None, True),
                         "GitHub-hosted runner, AMD EPYC 9V74")


class TwoPlatforms(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = tempfile.TemporaryDirectory()
        cls.linux_report = linux_shaped_copy(Path(cls.directory.name))
        cls.reports = {"macos-arm64": ROUND_4, "linux-x86-64": cls.linux_report}
        cls.platforms = [publisher.measure(key, path) for key, path in cls.reports.items()]

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def test_a_report_filed_under_the_wrong_platform_is_refused(self):
        with self.assertRaises(SystemExit):
            publisher.measure("macos-arm64", self.linux_report)
        with self.assertRaises(SystemExit):
            publisher.measure("linux-x86-64", ROUND_4)

    def test_a_missing_report_is_refused(self):
        with self.assertRaises(SystemExit):
            publisher.measure("linux-x86-64", Path("docs/reports/0000-00-00-native-missing"))

    def test_the_linux_platform_carries_the_shared_runner_caveat(self):
        linux = self.platforms[1]
        self.assertEqual(linux["machine"], "GitHub-hosted runner, AMD EPYC 7763")
        self.assertTrue(linux["sharedRunner"])
        self.assertEqual(linux["files"], ["PROVENANCE.md"])

    def test_the_readme_states_each_platform_separately_without_a_table(self):
        block = publisher.readme_block(self.platforms)
        self.assertTrue(block.startswith(publisher.README_START + "\n"))
        self.assertTrue(block.endswith("\n" + publisher.README_END))
        text = " ".join(block.split())
        self.assertIn("on Apple Silicon (Apple M1 Pro, fresh parser state, 2026-09-21)", text)
        self.assertIn("On Linux x86-64 (GitHub-hosted runner, AMD EPYC 7763, 2026-09-24)", text)
        self.assertEqual(text.count(publisher.CI_CLAIMS), 1)
        # The README contract forbids tables in README.md.src.
        self.assertFalse(any(line.startswith("|") for line in block.splitlines()))

    def test_the_readme_never_claims_faster_where_the_archive_does_not(self):
        platforms = copy.deepcopy(self.platforms)
        platforms[1]["figures"][0]["fresh"] = 0.98
        with self.assertRaises(SystemExit):
            publisher.readme_block(platforms)

    def test_the_homepage_figures_list_both_platforms_in_publishing_order(self):
        website = json.loads(publisher.website_json(self.platforms))
        self.assertEqual([p["id"] for p in website["platforms"]], ["macos-arm64", "linux-x86-64"])
        self.assertEqual([p["sharedRunner"] for p in website["platforms"]], [False, True])

    def test_the_guide_shows_both_tables_and_links_each_report(self):
        section = publisher.guide_section(self.platforms)
        self.assertTrue(section.startswith(publisher.GUIDE_START))
        self.assertEqual(section.count("| Engine | Fresh, 14 agreeing across six |"), 2)
        for platform in self.platforms:
            self.assertIn(f"### {platform['label']}", section)
            self.assertIn(publisher.link(platform["report"] + "/README.md"), section)
        self.assertIn(publisher.CI_CLAIMS, section)
        # The published report is not listed again among the earlier ones.
        earlier = section.split("### Earlier comparisons")[1]
        self.assertNotIn(ROUND_4.as_posix(), earlier)
        self.assertIn("2026-09-21-native-release-fixed", earlier)

    def test_the_guide_stays_valid_mdx(self):
        section = publisher.guide_section(self.platforms)
        for character in "<>{}":
            self.assertNotIn(character, section)

    def test_wrapping_never_splits_a_link_text(self):
        wrapped = publisher.fill("word " * 30 + "[held-out PGO comparison](https://example.invalid/a) end")
        self.assertIn("[held-out PGO comparison](", wrapped)


if __name__ == "__main__":
    unittest.main()
