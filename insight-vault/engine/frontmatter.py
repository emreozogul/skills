"""Parse, serialize, and validate insight frontmatter.

Constrained YAML subset (stdlib-only, no PyYAML):
- delimited by lines that are exactly '---'
- scalar fields:        key: value
- inline list:          tags: [a, b, c]
- block list:           key:\n  - item\n  - item
Relations are strings: "<type> <to_id> [:: note]".
"""
import re

REQUIRED = ["id", "title", "domain", "type", "confidence",
            "created", "updated", "status", "source_kind", "source_ref"]
TYPES = {"finding", "stat", "principle", "observation", "prediction", "quote"}
CONFIDENCE = {"high", "medium", "low", "speculative"}
STATUS = {"active", "superseded", "archived"}
SOURCE_KIND = {"url", "document", "conversation", "research", "manual"}
RELATION_TYPES = {"supports", "contradicts", "refines", "duplicates", "supersedes"}

KEY_ORDER = ["id", "title", "domain", "tags", "type", "confidence",
             "source_kind", "source_ref", "source_date",
             "created", "updated", "status", "relations"]

ID_RE = re.compile(r"^\d{8}-[0-9a-f]{4,8}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
DOMAIN_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")


def _unquote(s):
    if len(s) >= 2 and s[0] == s[-1] and s[0] in "\"'":
        return s[1:-1]
    return s


def parse(text):
    """Return (meta: dict, body: str)."""
    lines = text.splitlines()
    start = None
    for i, ln in enumerate(lines):
        if ln.strip() == "":
            continue
        if ln.strip() == "---":
            start = i
        break
    if start is None:
        return {}, text
    end = None
    for j in range(start + 1, len(lines)):
        if lines[j].strip() == "---":
            end = j
            break
    if end is None:
        return {}, text
    fm_lines = lines[start + 1:end]
    body = "\n".join(lines[end + 1:]).lstrip("\n")
    meta = {}
    i = 0
    while i < len(fm_lines):
        line = fm_lines[i]
        if not line.strip():
            i += 1
            continue
        key, _, rest = line.partition(":")
        key = key.strip()
        rest = rest.strip()
        if rest == "":
            items = []
            j = i + 1
            while j < len(fm_lines) and fm_lines[j].lstrip().startswith("- "):
                items.append(fm_lines[j].lstrip()[2:].strip())
                j += 1
            if items:
                meta[key] = items
                i = j
            else:
                meta[key] = ""
                i += 1
        elif rest.startswith("[") and rest.endswith("]"):
            inner = rest[1:-1].strip()
            meta[key] = [x.strip() for x in inner.split(",")] if inner else []
            i += 1
        else:
            meta[key] = _unquote(rest)
            i += 1
    return meta, body


def serialize(meta, body):
    """Return full markdown file text for meta + body."""
    out = ["---"]
    keys = [k for k in KEY_ORDER if k in meta] + [k for k in meta if k not in KEY_ORDER]
    for k in keys:
        v = meta[k]
        if isinstance(v, list):
            if k == "relations":
                if v:
                    out.append(f"{k}:")
                    out.extend(f"  - {item}" for item in v)
                else:
                    out.append(f"{k}: []")
            else:
                out.append(f"{k}: [{', '.join(str(x) for x in v)}]")
        else:
            out.append(f"{k}: {v}")
    out.append("---")
    out.append("")
    out.append(body.rstrip("\n"))
    out.append("")
    return "\n".join(out)


def validate(meta):
    """Return a list of error strings (empty list == valid)."""
    errors = []
    for key in REQUIRED:
        if not meta.get(key):
            errors.append(f"missing required field: {key}")
    if meta.get("type") and meta["type"] not in TYPES:
        errors.append(f"invalid type: {meta['type']}")
    if meta.get("confidence") and meta["confidence"] not in CONFIDENCE:
        errors.append(f"invalid confidence: {meta['confidence']}")
    if meta.get("status") and meta["status"] not in STATUS:
        errors.append(f"invalid status: {meta['status']}")
    if meta.get("source_kind") and meta["source_kind"] not in SOURCE_KIND:
        errors.append(f"invalid source_kind: {meta['source_kind']}")
    if meta.get("id") and not ID_RE.match(str(meta["id"])):
        errors.append(f"invalid id format: {meta['id']}")
    if meta.get("domain") and not DOMAIN_RE.match(str(meta["domain"])):
        errors.append(f"invalid domain (must be a lowercase slug): {meta['domain']}")
    for d in ("created", "updated", "source_date"):
        if meta.get(d) and not DATE_RE.match(str(meta[d])):
            errors.append(f"invalid date format for {d}: {meta[d]}")
    return errors


def parse_relation(s):
    """'contradicts 20260512-b1c2 :: note' -> (type, to_id, note)."""
    note = ""
    if " :: " in s:
        s, note = s.split(" :: ", 1)
    parts = s.split()
    rtype = parts[0] if parts else ""
    to_id = parts[1] if len(parts) > 1 else ""
    return rtype, to_id, note.strip()


def format_relation(rtype, to_id, note=""):
    base = f"{rtype} {to_id}"
    return f"{base} :: {note}" if note else base
