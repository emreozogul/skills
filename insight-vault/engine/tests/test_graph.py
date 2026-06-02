import os, sys, tempfile, unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
import db, graph, frontmatter as fm


def _meta(iid, domain="d", tags=None, rels=None):
    return {
        "id": iid, "title": f"Title {iid}", "domain": domain,
        "tags": tags or [], "type": "finding", "confidence": "high",
        "status": "active", "source_kind": "manual", "source_ref": "t",
        "created": "2026-06-01", "updated": "2026-06-01", "relations": rels or [],
    }


class TestGraph(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.conn = db.connect(os.path.join(self.tmp, "g.db"))
        self.addCleanup(self.conn.close)
        # A: pricing/saas ; B: pricing/saas (shares 2 tags w/ A) ; C: linked to A supports
        db.upsert(self.conn, _meta("20260601-aaaa", "pricing", ["pricing", "saas"]), "## Claim\nA", "/a.md")
        db.upsert(self.conn, _meta("20260601-bbbb", "pricing", ["pricing", "saas"]), "## Claim\nB", "/b.md")
        db.upsert(self.conn, _meta("20260601-cccc", "product", ["churn"],
                  ["supports 20260601-aaaa :: c backs a"]), "## Claim\nC", "/c.md")
        db.upsert(self.conn, _meta("20260601-dddd", "product", ["orphan-tag"]), "## Claim\nD", "/d.md")
        self.conn.commit()

    def test_neighbors_direct(self):
        nb = graph.neighbors(self.conn, "20260601-aaaa")
        ids = {n["id"] for n in nb}
        self.assertIn("20260601-cccc", ids)  # C supports A -> A neighbors C

    def test_neighbors_depth_and_type_filter(self):
        only = graph.neighbors(self.conn, "20260601-aaaa", types=["supports"])
        self.assertTrue(all(n["type"] == "supports" for n in only))
        none = graph.neighbors(self.conn, "20260601-aaaa", types=["contradicts"])
        self.assertEqual(none, [])

    def test_path_between(self):
        p = graph.path(self.conn, "20260601-cccc", "20260601-aaaa")
        self.assertEqual(p, ["20260601-cccc", "20260601-aaaa"])

    def test_path_none(self):
        self.assertIsNone(graph.path(self.conn, "20260601-dddd", "20260601-aaaa"))

    def test_orphans(self):
        o = set(graph.orphans(self.conn))
        self.assertIn("20260601-dddd", o)       # D has no edges
        self.assertNotIn("20260601-aaaa", o)    # A is linked

    def test_contradictions_empty_then_present(self):
        self.assertEqual(graph.contradictions(self.conn), [])
        db.upsert(self.conn, _meta("20260601-eeee", "d", [],
                  ["contradicts 20260601-aaaa :: tension"]), "## Claim\nE", "/e.md")
        self.conn.commit()
        c = graph.contradictions(self.conn)
        self.assertEqual(len(c), 1)
        self.assertEqual({c[0]["from"], c[0]["to"]}, {"20260601-eeee", "20260601-aaaa"})

    def test_components(self):
        comps = graph.components(self.conn)
        # A,B,C connected via A; wait B is only tag-adjacent, not edge-linked -> B & D are singletons
        sizes = sorted(len(c) for c in comps)
        self.assertIn(2, sizes)   # A-C component
        self.assertIn(1, sizes)   # D singleton

    def test_suggest_links_shared_tags(self):
        sugg = graph.suggest_links(self.conn, "20260601-aaaa")
        ids = {s["id"] for s in sugg}
        self.assertIn("20260601-bbbb", ids)         # shares pricing+saas, no edge yet
        self.assertNotIn("20260601-cccc", ids)      # already linked -> excluded
        self.assertNotIn("20260601-aaaa", ids)      # not self

    def test_graph_data_shape(self):
        g = graph.graph_data(self.conn)
        self.assertEqual(len(g["nodes"]), 4)
        self.assertTrue(any(e["type"] == "supports" for e in g["edges"]))
        node = next(n for n in g["nodes"] if n["id"] == "20260601-aaaa")
        self.assertIn("domain", node)
        self.assertIn("title", node)

    def test_mermaid_render(self):
        m = graph.to_mermaid(self.conn)
        self.assertIn("graph LR", m)
        self.assertIn("20260601_aaaa", m)  # ids sanitized for mermaid (no hyphen)


if __name__ == "__main__":
    unittest.main()
