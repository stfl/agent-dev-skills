#!/usr/bin/env python3
"""Comment density per file, and the comment lines most likely to rot.

Usage: comments.py [--diff RANGE] [--tests] [FILE ...]

  FILE ...        files to measure (default: every tracked source file)
  --diff RANGE    count only the lines RANGE adds, e.g. origin/main...HEAD
  --tests         include Rust test modules (from `#[cfg(test)]` on), skipped by default

Prints doc, inline and code lines per file, densest first, then every comment
line that states a count, a copied value, a history, or another file's name:
the facts that go stale while the code beside them changes.
"""

import re
import subprocess
import sys
from pathlib import Path

SLASH = {
    ".rs",
    ".c",
    ".h",
    ".cc",
    ".cpp",
    ".hpp",
    ".go",
    ".js",
    ".ts",
    ".tsx",
    ".java",
    ".kt",
    ".swift",
    ".zig",
}
HASH = {".py", ".sh", ".bash", ".toml", ".nix", ".yaml", ".yml", ".rb", ".pl"}
HASH_NAMES = {"justfile", "Justfile", "Makefile", "Dockerfile"}

NUMBER = r"two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|twenty|thirty|forty|fifty|sixty|hundred|both|twice"
ROT = [
    ("count", re.compile(rf"\b({NUMBER})\b|\b\d+ of \d+\b", re.I)),
    (
        "value",
        re.compile(
            r"(?<![`\w.])\d[\d_.,]*\s*(ms|s|min|h|KiB|MiB|GiB|kB|MB|bytes?|%|seconds?|minutes?|hours?|lines?|bins?)?\b"
        ),
    ),
    (
        "history",
        re.compile(
            r"\b(no longer|previously|used to|formerly|newly|anymore|was changed|instead of the old)\b",
            re.I,
        ),
    ),
    (
        "xref",
        re.compile(r"\b[\w-]+\.(rs|py|ts|go|c|h|toml|md)\b|\bline \d+\b|\bL\d+\b"),
    ),
]


def syntax(path):
    p = Path(path)
    if p.suffix in SLASH:
        return "//"
    if p.suffix in HASH or p.name in HASH_NAMES:
        return "#"
    return None


def classify(line, marker):
    s = line.strip()
    if not s:
        return None
    if marker == "//":
        if s.startswith(("///", "//!")):
            return "doc"
        if s.startswith("//"):
            return "inline"
    elif s.startswith("#") and not s.startswith(("#!", "#[")):
        return "inline"
    return "code"


def comment_text(line):
    return re.sub(r"^\s*(//[/!]?|#)\s?", "", line)


def git(*args):
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True
    ).stdout


def added_lines(rng):
    """{path: set of line numbers RANGE adds}, numbered as on its right side."""
    out, path = {}, None
    for line in git("diff", "-U0", "--no-color", rng).splitlines():
        if line.startswith("+++ "):
            path = line[6:] if line.startswith("+++ b/") else None
        elif line.startswith("@@") and path:
            m = re.search(r"\+(\d+)(?:,(\d+))?", line)
            start, n = int(m.group(1)), int(m.group(2) or 1)
            out.setdefault(path, set()).update(range(start, start + n))
    return out


def right_side(rng):
    if "..." in rng:
        return rng.split("...")[1] or "HEAD"
    if ".." in rng:
        return rng.split("..")[1] or "HEAD"
    return None  # a single rev diffs against the working tree


def measure(path, text, only, tests):
    marker = syntax(path)
    lines = text.splitlines()
    if marker == "//" and path.endswith(".rs") and not tests:
        if "/tests/" in path:
            return None
        cut = next(
            (
                i
                for i, l in enumerate(lines[:-1])
                if l.strip() == "#[cfg(test)]"
                and lines[i + 1].lstrip().startswith("mod ")
            ),
            len(lines),
        )
        lines = lines[:cut]
    counts, hits = {"doc": 0, "inline": 0, "code": 0}, []
    for no, line in enumerate(lines, 1):
        if only is not None and no not in only:
            continue
        kind = classify(line, marker)
        if kind is None:
            continue
        counts[kind] += 1
        if kind != "code":
            body = comment_text(line)
            tags = [tag for tag, rx in ROT if rx.search(body)]
            if tags:
                hits.append((no, ",".join(tags), body.strip()))
    return counts, hits


def main(argv):
    rng, tests, files = None, False, []
    it = iter(argv)
    for a in it:
        if a == "--diff":
            rng = next(it)
        elif a == "--tests":
            tests = True
        else:
            files.append(a)
    added = added_lines(rng) if rng else {}
    rev = right_side(rng) if rng else None
    if not files:
        files = sorted(added) if rng else git("ls-files").split()
    files = [f for f in files if syntax(f)]

    rows, rot = [], []
    for f in files:
        if rng and f not in added:
            continue
        text = git("show", f"{rev}:{f}") if rev else Path(f).read_text(errors="replace")
        res = measure(f, text, added.get(f) if rng else None, tests)
        if res is None:
            continue
        counts, hits = res
        if sum(counts.values()):
            rows.append((f, counts))
        rot += [(f, *h) for h in hits]

    rows.sort(key=lambda r: -(r[1]["doc"] + r[1]["inline"]))
    print(
        f"{'doc':>5} {'//':>5} {'code':>5} {'ratio':>6}  file"
        + (f"   (lines added by {rng})" if rng else "")
    )
    tot = {"doc": 0, "inline": 0, "code": 0}
    for f, c in rows:
        for k in tot:
            tot[k] += c[k]
        print(
            f"{c['doc']:5} {c['inline']:5} {c['code']:5} {(c['doc'] + c['inline']) / max(c['code'], 1):6.2f}  {f}"
        )
    print(
        f"{tot['doc']:5} {tot['inline']:5} {tot['code']:5} {(tot['doc'] + tot['inline']) / max(tot['code'], 1):6.2f}  TOTAL"
    )

    # A candidate, not a verdict: "both" in "both directions" is fine, and a
    # spec number is a link. Read each; keep the ones the code cannot state.
    if rot:
        print(
            f"\nRot candidates ({len(rot)}): counts, copied values, history, file names"
        )
        for f, no, tags, body in rot:
            print(f"{f}:{no}  [{tags}]  {body[:110]}")


if __name__ == "__main__":
    main(sys.argv[1:])
