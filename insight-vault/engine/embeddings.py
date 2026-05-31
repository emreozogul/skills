"""Pluggable embedder interface. Default is a no-op; semantic search is off.

To enable semantic search later, add a real Embedder subclass here and return
it from get_embedder(), then add an `embeddings` population step in the CLI.
No schema change is needed — the `embeddings` table already exists.
"""


class Embedder:
    name = "base"

    def embed(self, texts):
        raise NotImplementedError


class NoopEmbedder(Embedder):
    name = "none"

    def embed(self, texts):
        return [None for _ in texts]


def get_embedder(name="none"):
    if name in ("none", "", None):
        return NoopEmbedder()
    raise ValueError(
        f"Unknown/unsupported embedder: {name!r} (semantic search not enabled)")
