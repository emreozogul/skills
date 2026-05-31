import io, json, os, sys, tempfile, unittest
from contextlib import redirect_stdout
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
import insight

ADD_DOC = """---
title: SaaS churn rises above 5% monthly
domain: market-research
tags: [pricing, saas, churn]
type: finding
confidence: high
source_kind: url
source_ref: https://example.com/report
source_date: 2026-05-30
---
## Claim
Monthly SaaS churn now exceeds five percent.

## Evidence
Cohort analysis across 40 companies.
"""


def run(argv):
    buf = io.StringIO()
    with redirect_stdout(buf):
        insight.main(argv)
    return json.loads(buf.getvalue().strip().splitlines()[-1])


class TestCli(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        os.environ["INSIGHT_VAULT"] = os.path.join(self.tmp, "vault")
        os.environ["INSIGHT_DB"] = os.path.join(self.tmp, "index", "i.db")
        self.doc = os.path.join(self.tmp, "in.md")
        with open(self.doc, "w", encoding="utf-8") as fh:
            fh.write(ADD_DOC)

    def tearDown(self):
        os.environ.pop("INSIGHT_VAULT", None)
        os.environ.pop("INSIGHT_DB", None)

    def test_add_then_search_then_get(self):
        added = run(["add", "--file", self.doc])
        self.assertTrue(added["ok"])
        iid = added["id"]
        self.assertTrue(os.path.exists(added["path"]))
        found = run(["search", "churn saas"])
        self.assertEqual(found["count"], 1)
        self.assertEqual(found["results"][0]["id"], iid)
        got = run(["get", iid])
        self.assertEqual(got["insight"]["domain"], "market-research")

    def test_add_rejects_invalid(self):
        bad = os.path.join(self.tmp, "bad.md")
        with open(bad, "w", encoding="utf-8") as fh:
            fh.write("---\ntitle: no domain\ntype: finding\n---\n## Claim\nx\n")
        with self.assertRaises(SystemExit):
            run(["add", "--file", bad])

    def test_link_writes_both_sides(self):
        a = run(["add", "--file", self.doc])["id"]
        doc2 = os.path.join(self.tmp, "in2.md")
        with open(doc2, "w", encoding="utf-8") as fh:
            fh.write(ADD_DOC.replace("rises above 5%", "is flat year over year"))
        b = run(["add", "--file", doc2])["id"]
        linked = run(["link", a, b, "--type", "contradicts", "--note", "cohorts differ"])
        self.assertTrue(linked["ok"])
        rec_a = run(["get", a])["insight"]
        rec_b = run(["get", b])["insight"]
        self.assertTrue(any(r["to_id"] == b for r in rec_a["relations"]))
        self.assertTrue(any(r["to_id"] == a for r in rec_b["relations"]))

    def test_stats_and_vault_path(self):
        run(["add", "--file", self.doc])
        st = run(["stats"])["stats"]
        self.assertEqual(st["total"], 1)
        vp = run(["vault-path"])
        self.assertTrue(vp["vault"].endswith("vault"))

    def test_add_rejects_path_traversal_domain(self):
        evil = os.path.join(self.tmp, "evil.md")
        with open(evil, "w", encoding="utf-8") as fh:
            fh.write(ADD_DOC.replace("domain: market-research", "domain: ../../../../tmp/evil"))
        with self.assertRaises(SystemExit):
            run(["add", "--file", evil])

    def test_raw_bad_fts_returns_json_error(self):
        run(["add", "--file", self.doc])
        buf = io.StringIO()
        with redirect_stdout(buf):
            with self.assertRaises(SystemExit):
                insight.main(["search", "alpha AND", "--raw"])
        out = json.loads(buf.getvalue().strip().splitlines()[-1])
        self.assertFalse(out["ok"])
        self.assertIn("error", out)

    def test_pretty_accepted_after_subcommand(self):
        run(["add", "--file", self.doc])
        buf = io.StringIO()
        with redirect_stdout(buf):
            insight.main(["search", "churn", "--pretty"])
        res = json.loads(buf.getvalue())
        self.assertTrue(res["ok"])

    def test_self_link_rejected(self):
        a = run(["add", "--file", self.doc])["id"]
        with self.assertRaises(SystemExit):
            run(["link", a, a, "--type", "supports"])


if __name__ == "__main__":
    unittest.main()
