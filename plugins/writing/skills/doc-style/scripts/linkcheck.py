#!/usr/bin/env python3
"""Resolve every relative markdown link and heading anchor in a doc set.

Usage: linkcheck.py [FILE_OR_DIR ...]     (default: README.md docs)

A split breaks links silently: the anchor a section used to own moves to another
file and nothing complains. Run this after moving anything, and again before
committing. Exits non-zero when something dangles.

GitHub-relative paths that resolve on the site but not on disk (../../issues,
../../releases, ../../pulls, ../../wiki) are skipped.
"""

import os
import re
import sys

GITHUB_RELATIVE = {"../../issues", "../../releases", "../../pulls", "../../wiki"}
LINK = re.compile(r"\]\(([^)\s]+)\)")
HEADING = re.compile(r"^#{1,6}\s+(.*)")


def slugs(path):
    """GitHub's heading -> anchor rule, closely enough for link checking."""
    found = set()
    for line in open(path, errors="replace"):
        m = HEADING.match(line)
        if not m:
            continue
        text = re.sub(r"[`*\[\]()]", "", m.group(1).strip()).lower()
        text = re.sub(r"[^a-z0-9 \-]", "", text)
        found.add(re.sub(r"-+", "-", text.strip().replace(" ", "-")))
    return found


def markdown_files(targets):
    for t in targets:
        if os.path.isdir(t):
            for root, _, names in os.walk(t):
                if ".git" in root or "/target" in root:
                    continue
                for n in sorted(names):
                    if n.endswith(".md"):
                        yield os.path.join(root, n)
        elif os.path.exists(t):
            yield t


def main(targets):
    broken = []
    for path in markdown_files(targets):
        base = os.path.dirname(path)
        for m in LINK.finditer(open(path, errors="replace").read()):
            target = m.group(1)
            if target.startswith(("http", "mailto", "#!")) or target in GITHUB_RELATIVE:
                continue
            rel, _, anchor = target.partition("#")
            resolved = os.path.normpath(os.path.join(base, rel)) if rel else path
            if rel and not os.path.exists(resolved):
                broken.append((path, target, "no such file"))
            elif anchor and resolved.endswith(".md") and os.path.exists(resolved):
                if anchor not in slugs(resolved):
                    broken.append((path, target, "no such heading"))
    for path, target, why in broken:
        print(f"{path}: {target} — {why}")
    print(f"{len(broken)} broken" if broken else "all links resolve")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:] or ["README.md", "docs"]))
