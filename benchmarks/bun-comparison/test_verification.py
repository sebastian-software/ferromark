import unittest

from run import CanonicalHTML, PARSERS, mismatches
from audit import groups


def canonical(html):
    return CanonicalHTML(html).tokens


class VerificationTests(unittest.TestCase):
    def test_every_timed_candidate_must_participate_in_parity(self):
        outputs = {name: "<p>ok</p>" for name in PARSERS}
        self.assertEqual(mismatches(outputs), [])
        outputs["md4c"] = "<p>different</p>"
        self.assertEqual(mismatches(outputs), ["md4c"])
        del outputs["md4c"]
        with self.assertRaises(ValueError):
            mismatches(outputs)

    def test_agreement_groups_do_not_treat_ferromark_as_an_oracle(self):
        outputs = {name: "<p>linked</p>" for name in PARSERS}
        outputs["ferromark"] = "<p>literal</p>"
        self.assertCountEqual(groups(outputs),
                              [["bun_md", "comrak", "md4c", "pulldown-cmark"], ["ferromark"]])

    def test_serialization_only_differences_are_accepted(self):
        self.assertEqual(canonical('<p>A &amp; B<br />C</p>\n'), canonical('<p>A &#38; B<br>C</p>'))
        self.assertEqual(canonical('<td align="right">1</td>'), canonical('<td style="text-align: right;">1</td>'))
        self.assertEqual(canonical('<input disabled="disabled" checked />'), canonical('<input checked="" disabled>'))

    def test_wrong_table_alignment_is_rejected(self):
        self.assertNotEqual(canonical('<td align="right">1</td>'), canonical('<td align="center">1</td>'))
        self.assertNotEqual(canonical('<td>1</td>'), canonical('<td align="left">1</td>'))

    def test_content_and_link_changes_are_rejected(self):
        self.assertNotEqual(canonical('<a href="/a">x</a>'), canonical('<a href="/b">x</a>'))
        self.assertNotEqual(canonical('<p>A &amp; B</p>'), canonical('<p>A &amp;amp; B</p>'))
        self.assertNotEqual(canonical('<p>[reference]</p>'), canonical('<p><a href="/a">reference</a></p>'))

    def test_literal_and_inline_whitespace_is_preserved(self):
        self.assertNotEqual(canonical('<pre><code>\n</code></pre>'), canonical('<pre><code>\n\n</code></pre>'))
        self.assertNotEqual(canonical('<p><em>a</em> <em>b</em></p>'), canonical('<p><em>a</em><em>b</em></p>'))
        self.assertEqual(canonical('<p>a \nb</p>'), canonical('<p>a\nb</p>'))

    def test_flow_whitespace_respects_block_and_inline_boundaries(self):
        self.assertEqual(canonical('<li>parent\n<ul><li>child</li></ul></li>'),
                         canonical('<li>parent<ul><li>child</li></ul></li>'))
        self.assertEqual(canonical('<p>a\n b</p>'), canonical('<p>a b</p>'))
        self.assertNotEqual(canonical('<p><em>a</em>\n<em>b</em></p>'),
                            canonical('<p><em>a</em><em>b</em></p>'))
        self.assertNotEqual(canonical('<p>a&nbsp;b</p>'), canonical('<p>a b</p>'))
        self.assertNotEqual(canonical('<code>a  b</code>'), canonical('<code>a b</code>'))
        self.assertNotEqual(canonical('<textarea>a  b</textarea>'), canonical('<textarea>a b</textarea>'))

    def test_markup_and_comments_are_preserved(self):
        self.assertNotEqual(canonical('<p>hi</p>'), canonical('<div>hi</div>'))
        self.assertNotEqual(canonical('<!-- a -->'), canonical('<!-- b -->'))


if __name__ == '__main__':
    unittest.main()
