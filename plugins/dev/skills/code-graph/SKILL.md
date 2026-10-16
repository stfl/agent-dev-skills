---
name: code-graph
description: "Review a diff with the code-review-graph CLI: build the repository's graph, let detect-changes rank the changed functions by risk and name their test gaps, and ask callers_of and tests_for before opening files. Use before a code review, or a review round, in a repository where the `code-review-graph` command is on PATH, and when told to use the code graph. The graph answers structure; correctness still comes from reading every hunk."
---

# Code graph

`code-review-graph` parses a repository with tree-sitter into a graph of its
functions, types, calls and tests. A review asks it what a diff touches and
what reaches it, and opens only the files that answer points to.

## Steps

1. **Build the graph** in the checkout you review:

   ```bash
   code-review-graph build --repo "$(git rev-parse --show-toplevel)"
   ```

   It takes seconds and writes only `.code-review-graph/`, which carries its
   own `*` `.gitignore`, so `git status --porcelain` does not change. A graph
   that exists already takes `update` instead of `build`.
2. **Rank the range.** `code-review-graph detect-changes --base <fixed point>
   --brief` summarises the changed functions, the flows they sit on, the test
   gaps and a risk score. Without `--brief` it prints JSON: read
   `review_priorities` as the order to review in, and `test_gaps` as changed
   code no test reaches.
3. **Ask before you read.** For each changed function of weight, its callers
   and its tests:

   ```bash
   code-review-graph query callers_of '<path>::<Type>.<function>'
   code-review-graph query tests_for '<path>::<Type>.<function>'
   ```

   A bare name that matches several nodes is refused with a disambiguation
   list; pass the qualified name. `code-review-graph impact --base <fixed
   point>` gives the blast radius when a public item changed.
4. **Read the hunks.** The graph says where to look, never whether the code is
   right: every changed line is still read. A test gap the graph names, on
   code whose behaviour the diff changes, is a finding in its own right.

## Limits

- A Rust method called through a field or a generic (`self.health.failed(now)`)
  may not resolve, and then `callers_of` lists only the tests. `grep` the
  method's name before you conclude it has no production caller.
- The graph is as old as its last `build` or `update`. Rebuild after a fetch
  that moved the branch.
