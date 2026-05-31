import os, sys, tempfile, unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
import db, frontmatter as fm

META = {
    "id": "20260530-a3f9", "title": "SaaS churn rises above 5% monthly",
    "domain": "market-research", "tags": ["pricing", "saas", "churn"],
    "type": "finding", "confidence": "high", "status": "active",
    "source_kind": "url", "source_ref": "https://example.com",
    "source_date": "2026-05-30", "created": "2026-05-30", "updated": "2026-05-30",
    "relations": [],
}
BODY = "## Claim\nMonthly SaaS churn now exceeds five percent.\n\n## Evidence\nData.\n"


def _vault(tmp):
    d = os.path.join(tmp, "market-research")
    os.makedirs(d, exist_ok=True)
    p = os.path.join(d, "20260530-a3f9--churn.md")
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(fm.serialize(META, BODY))
    return p


class TestDb(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.conn = db.connect(os.path.join(self.tmp, "i.db"))
        self.addCleanup(self.conn.close)

    def test_fts5_available(self):
        self.assertTrue(db.has_fts5(self.conn))

    def test_upsert_and_search(self):
        db.upsert(self.conn, META, BODY, "/x/churn.md")
        self.conn.commit()
        res = db.search(self.conn, "churn saas")
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0]["id"], "20260530-a3f9")
        self.assertIn("saas", res[0]["tags"])

    def test_search_filters_by_domain_and_tag(self):
        db.upsert(self.conn, META, BODY, "/x/churn.md")
        self.conn.commit()
        self.assertEqual(len(db.search(self.conn, "churn", domain="market-research")), 1)
        self.assertEqual(len(db.search(self.conn, "churn", domain="product")), 0)
        self.assertEqual(len(db.search(self.conn, "churn", tag="saas")), 1)
        self.assertEqual(len(db.search(self.conn, "churn", tag="nope")), 0)

    def test_search_handles_punctuation_query(self):
        db.upsert(self.conn, META, BODY, "/x/churn.md")
        self.conn.commit()
        # Raw FTS would choke on this; fts_query sanitizes it.
        self.assertEqual(len(db.search(self.conn, "churn?! (saas)")), 1)

    def test_rebuild_from_vault_is_idempotent(self):
        _vault(self.tmp)
        n1 = db.rebuild(self.conn, self.tmp)
        n2 = db.rebuild(self.conn, self.tmp)
        self.assertEqual((n1, n2), (1, 1))
        self.assertEqual(db.stats(self.conn)["total"], 1)

    def test_get_returns_record(self):
        db.upsert(self.conn, META, BODY, "/x/churn.md")
        rec = db.get(self.conn, "20260530-a3f9")
        self.assertEqual(rec["domain"], "market-research")
        self.assertEqual(rec["tags"], ["churn", "pricing", "saas"])


if __name__ == "__main__":
    unittest.main()
