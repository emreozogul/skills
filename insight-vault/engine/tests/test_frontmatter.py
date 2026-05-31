import os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
import frontmatter as fm


class TestFrontmatter(unittest.TestCase):
    META = {
        "id": "20260530-a3f9", "title": "SaaS churn rises above 5% monthly",
        "domain": "market-research", "tags": ["pricing", "saas", "churn"],
        "type": "finding", "confidence": "high",
        "source_kind": "url", "source_ref": "https://example.com/report",
        "source_date": "2026-05-30", "created": "2026-05-30",
        "updated": "2026-05-30", "status": "active",
        "relations": ["contradicts 20260512-b1c2 :: newer cohort data"],
    }
    BODY = "## Claim\nChurn exceeds 5%.\n\n## Evidence\nCohort table.\n"

    def test_round_trip(self):
        text = fm.serialize(self.META, self.BODY)
        meta, body = fm.parse(text)
        self.assertEqual(meta["id"], self.META["id"])
        self.assertEqual(meta["tags"], ["pricing", "saas", "churn"])
        self.assertEqual(meta["relations"], self.META["relations"])
        self.assertIn("Churn exceeds 5%", body)

    def test_title_with_colon(self):
        meta, _ = fm.parse(fm.serialize({**self.META, "title": "AI: a market"}, self.BODY))
        self.assertEqual(meta["title"], "AI: a market")

    def test_url_value_preserved(self):
        meta, _ = fm.parse(fm.serialize(self.META, self.BODY))
        self.assertEqual(meta["source_ref"], "https://example.com/report")

    def test_validate_ok(self):
        self.assertEqual(fm.validate(self.META), [])

    def test_validate_catches_bad_enum_and_missing(self):
        bad = {**self.META, "type": "nonsense"}
        del bad["title"]
        errs = fm.validate(bad)
        self.assertTrue(any("type" in e for e in errs))
        self.assertTrue(any("title" in e for e in errs))

    def test_parse_relation(self):
        self.assertEqual(fm.parse_relation("contradicts 20260512-b1c2 :: x"),
                         ("contradicts", "20260512-b1c2", "x"))
        self.assertEqual(fm.parse_relation("supports 20260101-aa11"),
                         ("supports", "20260101-aa11", ""))

    def test_no_frontmatter_returns_empty(self):
        meta, body = fm.parse("just text, no fence")
        self.assertEqual(meta, {})
        self.assertEqual(body, "just text, no fence")


if __name__ == "__main__":
    unittest.main()
