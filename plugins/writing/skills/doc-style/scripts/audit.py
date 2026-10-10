#!/usr/bin/env python3
"""Word budget per top-level section of a markdown document.

Usage: audit.py [FILE ...]        (default: README.md)

Prints one row per `##` section, heaviest first, with each section's share of
the document. The share column is the argument: a section holding 40% of a
README is the thing to move, whatever it is about.
"""

import re
import sys


def sections(path):
    """Split a markdown file at `##` headings. Returns (name, line, words)."""
    out, name, start, body = [], "(preamble)", 1, []
    for lineno, line in enumerate(open(path, errors="replace").read().split("\n"), 1):
        if re.match(r"^## ", line):
            out.append((name, start, len(" ".join(body).split())))
            name, start, body = line[3:].strip(), lineno, []
        body.append(line)
    out.append((name, start, len(" ".join(body).split())))
    return out


def report(path):
    secs = sections(path)
    total = sum(w for _, _, w in secs) or 1
    print(f"\n{path} — {total} words, {sum(1 for _ in open(path))} lines")
    print(f"{'words':>6} {'share':>6}  {'line':<7} section")
    heavy = []
    for name, line, words in sorted(secs, key=lambda s: -s[2]):
        # The preamble is the hook, the pitch and the highlights. It is the one
        # part that never moves, however heavy — so it is never a candidate.
        dominant = words / total > 0.25 and name != "(preamble)"
        if dominant:
            heavy.append(name)
        print(
            f"{words:6} {words / total:6.0%}  L{line:<6} {name}{' <- dominant' if dominant else ''}"
        )

    # Concentration is the smell, not size. One section owning a third of the
    # file is a reference manual in hiding; the same words spread evenly across
    # a primer's own sections are just a thorough primer.
    if heavy:
        print(f"\nDominant: {', '.join(heavy)} — route by reader, then move whole.")
    if total > 2000:
        print(f"\n{total} words, well past the ~1200 primer budget. Read it through:")
        print("what is the primer here, and what is reference wearing its filename?")
    elif total > 1200 and not heavy:
        print(f"\n{total} words, a little over the ~1200 guideline, with no section")
        print("dominating. Spread evenly across primer sections, this is fine.")


if __name__ == "__main__":
    for arg in sys.argv[1:] or ["README.md"]:
        report(arg)
