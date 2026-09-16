"""Exercise the release-body extraction without touching GitHub."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('finish', Path(__file__).with_name('finish-github-release.py'))
finish = importlib.util.module_from_spec(spec)
spec.loader.exec_module(finish)

CHANGELOG = """# Changelog

## [2.0.0-rc.2](https://github.com/sebastian-software/ferromark/compare/v2.0.0-rc.1...v2.0.0-rc.2) (2026-09-16)


### Performance Improvements

* allocate heading scratch buffers on first use ([0958933](https://github.com/sebastian-software/ferromark/commit/0958933))

## 2.0.0-rc.1

First release candidate.

## Install

```sh
npm install ferromark@next
```
"""


class ChangelogSectionTests(unittest.TestCase):
    def test_extracts_the_generated_section_without_its_heading(self):
        body = finish.changelog_section(CHANGELOG, '2.0.0-rc.2')
        self.assertTrue(body.startswith('### Performance Improvements'))
        self.assertIn('allocate heading scratch buffers', body)
        self.assertNotIn('2.0.0-rc.1', body)
        self.assertTrue(body.endswith('\n'))

    def test_accepts_the_authored_plain_heading_and_stops_at_the_next_section(self):
        body = finish.changelog_section(CHANGELOG, '2.0.0-rc.1')
        self.assertEqual(body, 'First release candidate.\n')

    def test_rejects_missing_or_empty_sections(self):
        with self.assertRaises(AssertionError):
            finish.changelog_section(CHANGELOG, '2.0.0-rc.3')
        with self.assertRaises(AssertionError):
            finish.changelog_section('# Changelog\n\n## 2.0.0\n\n## 1.9.0\n\nOld.\n', '2.0.0')

    def test_does_not_match_a_longer_version_prefix(self):
        with self.assertRaises(AssertionError):
            finish.changelog_section(CHANGELOG, '2.0.0-rc')


if __name__ == '__main__':
    unittest.main()
