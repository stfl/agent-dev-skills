---
name: doc-style
description: Keep a repo's documentation lean and routed to its reader — the README a human primer, docs/ the human reference, CLAUDE.md/AGENTS.md the agent notes, and comments in code only what the code cannot say. Use when writing, rewriting, reviewing, trimming or splitting a README; when a README has grown into a reference manual or "is a lot of prose nobody reads"; when asked to make docs human-facing, to write one as a developer advocate or for clients, to set up a docs/ layout, or to decide which file a fact belongs in. Also use when code carries more comments than code, when asked to slim, trim, clean up or review doc comments, rustdoc, docstrings or inline comments, when comments drift out of sync with the code (counts, copied values, history), or before answering a review question by adding a sentence to a comment. Also use before adding a section to an existing README, and before a branch that adds documentation lines or comments lands.
---

# Doc style

Documentation fails two ways, and length is a symptom of both. It serves the
wrong reader: a README nobody reads is three documents wearing one filename.
Or it repeats the code: a comment that restates a name, a count or a constant
is a second copy that the compiler never checks, and it drifts the first time
the code moves. Two rules state the policy — `readme-vs-agent-docs` routes
documents by reader and loads when a README, `CLAUDE.md` or `AGENTS.md` is
opened; `doc-wording` keeps each sentence true and is always loaded. This skill
executes them on a repo that already went wrong: **part A** restructures the
documents, **part B** slims the comments in the code.

# Part A — Documents

Restructuring is a **move**, not a rewrite. Cut and paste every displaced
section verbatim, land it, repair the links, and only then rewrite the primer
that remains. A reviewer can check a move; nobody can check a 6,000-word
simultaneous move-and-rewrite, and the temptation to "improve while moving" is
how facts quietly die.

## A1. Measure before deciding

```bash
python3 <skill>/scripts/audit.py README.md
```

One row per `##`, heaviest first, with each section's share. Argue from the
share: *"Configuration is 47% of this file"* settles a restructure that *"the
README feels long"* cannot. Anything over 15% is a candidate to move; a total
over ~1,200 words means the primer is already buried.

Read the whole document before moving anything. The audit finds the mass; only
reading finds the primer hiding inside it — usually intact, usually the original
version, with the manual stapled to its back.

## A2. Decide destinations

Route every heavy section by its reader, per `readme-vs-agent-docs`. The one judgement call
that recurs: **detail is not the test**. A flag table is dense and belongs in
`docs/`, because an operator needs it and will not open `CLAUDE.md`. A thread
model is dense and belongs in the agent notes, because only someone changing the
code needs it.

Give `docs/` one page per question a user arrives with, named for the question:
`configuration.md`, `triggers-and-time.md`, `operating.md`. A page may be short.
Five honest pages beat one `reference.md` that reproduces the original problem
one directory down.

## A3. Move

Per displaced section, in order:

1. Extract the line range verbatim.
2. Promote headings one level (`###` → `##`) so the page has a title.
3. Write a two-sentence header: what this page answers, and a link back.
4. Rewrite cross-links. An anchor that pointed within the README is now either
   same-page (keep it) or cross-file (`configuration.md#topics`). Relative paths
   gain a `../`.
5. Reword deixis. "See below" and "the tables above" are false across a file
   boundary — name the target instead.

## A4. Repair

```bash
python3 <skill>/scripts/linkcheck.py README.md docs
grep -rn "README.md#" . --exclude-dir=.git --exclude-dir=target
grep -rnE '\]\([^)]*((CLAUDE|AGENTS)\.md|\.claude/)' README.md CONTRIBUTING.md docs/
```

The second command is the one that catches real damage: every *inbound* link
from sibling docs, crate notes, examples, and source comments that named an
anchor which just moved. Rustdoc and docstrings cite READMEs too — grep the
source, not only the markdown. Expect to find an anchor that was already broken
before you started; fix it while you are there.

The third catches the direction the rule forbids — a human-facing page sending
its reader into the agent notes. Deleting the link is rarely the fix, because the
fact it reached for has no human-facing home; that is what the hit is telling
you. Give the fact one (a `docs/` page, or `CONTRIBUTING.md` when it is about
changing the code) and point there. A repo whose README routes contributors into
`CLAUDE.md` is missing a `CONTRIBUTING.md`, not missing a better link.

## A5. Rewrite the primer

Now, and only now. Canonical section order, above-the-fold rules, exemplars and
anti-patterns with sources: `references/readme-reference.md`. The four moves
that carry the most weight:

- **Hoist the proof.** The strongest verifiable claim — a benchmark, a binary
  size, a one-line integration — goes in the opening bullets, not in a section
  below the fold. Sell by demonstration: a real number or a real command, never
  an adjective. "0.45% of one core" is marketing; "blazing fast" is noise.
- **Show output.** A quickstart that runs three commands and then *asserts* a
  result proves nothing. Print what comes back.
- **State the limits.** Name what the project does not do, in its own section.
  This reads as the opposite of marketing and works as the best of it — ripgrep
  ships a "Why shouldn't I use ripgrep?" section for exactly this reason.
- **Verify every support claim.** Badges rot silently. Check the distro,
  platform and version list against the CI matrix and the release matrix before
  republishing it, and check the architecture the release artefacts are actually
  built for. A badge advertising a platform that cannot compile is worse than no
  badge.

## A6. Seal it

A README regrows unless the repo says what earns a place in it. Add to the
repo's own `CLAUDE.md`: the `docs/` layer with one line per page, and the
admission test — *a new fact earns a place in the README only by changing what
the project is or what it costs to run; everything else goes to `docs/` and is
linked.* Without this the next contributor appends, and the audit reads the same
in six months.

# Part B — Comments in code

A comment earns its line by saying what the code cannot: why, an invariant no
type enforces, a constraint from outside the code. The rules are `doc-wording`'s;
the reasoning and sources are in `references/code-comments.md`. Slimming is a
**tidying**, never mixed with a behaviour change, so the diff is reviewable as
comments only.

## B1. Measure

```bash
python3 <skill>/scripts/comments.py                              # whole repo
python3 <skill>/scripts/comments.py --diff origin/main...HEAD    # what a branch adds
```

Doc, inline and code lines per file, densest first, production code only, then
the **rot candidates**: comment lines stating a count, a value, a history, or
another file's name. Argue from the ratio: *"stream.rs carries 0.55 comment
lines per line of code"* starts the work that *"there is a lot of doc"* cannot.
Past about 0.5, read the file; the comments are holding something that belongs
elsewhere. The candidate list is a reading list, not a verdict.

## B2. Decide each comment

| Verdict | When | Do |
|---|---|---|
| **Keep** | a why; an invariant no type enforces; a constraint from outside (protocol behaviour, an upstream bug, a unit); a `SAFETY:` argument | Tighten it to the one sentence that carries it |
| **Cut** | restates the name, the signature, the type or the next line; narrates the steps | Delete. Do not reword |
| **Link** | a count, a list of members or callers, a copied value | Name the item (Rust: an intra-doc link) or the property the members share |
| **Move** | design narrative, rejected alternatives, history, operator facts | Spec or ADR; commit message; `docs/`. Leave one link, or none |
| **Encode** | a claim the code could enforce | A type, a name, an assertion or a test; then cut the comment |

A comment the branch moved to a spec must already be said there; check before
deleting, and add the missing sentence to the spec rather than lose it.

## B3. Shape what stays

- **Module header** (`//!`, a file docstring): what the module is for, the
  invariants it keeps, one link to the spec that decided its shape. A header
  past a screen is the spec in disguise; link the spec instead.
- **Item doc**: the first sentence says what a caller cannot infer from the
  signature. Nothing for an item whose name and type already say it.
- **Inline comment**: beside a line that looks wrong and is not.
- **Tests**: the test name is the comment.

## B4. Answer reviews in code

A review question answered by adding a sentence to a doc comment is how a
header grows a paragraph per round. Answer with a rename, a type, a test or a
spec line first. Add the sentence only when the answer is a why nothing else
can carry, and then it replaces a sentence rather than joining them.

## B5. Edit, then prove only comments changed

One commit per module or crate, apart from any behaviour change. Before each:

```bash
git diff -U0 -- '*.rs' | grep -E '^[+-]' | grep -vE '^(\+\+\+|---) ' \
  | grep -vE '^[+-][[:space:]]*(//|$)'     # must print nothing
```

Then the build's doc checks — for Rust, the docs build with warnings denied
(broken intra-doc links) and the doctests — and B1 again. Report the before
and after totals.

## B6. Seal it

Comments regrow too. Put the rule in the repo's own agent notes, in a few
lines a contributor without these global rules can follow: what a comment
holds, the no-counts and no-copied-values rule, and where reasoning goes.

# Claims a line makes

A line of a document or a comment that states a rule or a claim about the
code is checked against the code before it lands, and before anyone is asked
to approve it: every line the branch adds, not only those of its last change.
Why: a rule that is false of the code in the commit that writes it reads as
settled fact to every later change.

- **An absolute word is a claim to falsify.** Grep the codebase for what each
  *every*, *only*, *never*, *none* or *the one place* denies, members and log
  calls at every level included, and narrow the word where the code
  disagrees. Each place such a word covers is a claim of its own, and so is
  each example after *such as* and each changed outcome a changelog lists:
  hold each with a test of its own, or narrow the line to those that have
  one. Check changed outcomes against every outcome the code tells apart: an
  input that moves from rejected to accepted, one that moves the other way,
  and one that keeps its result under another option. Why: a test of one
  place leaves the others unchecked.
- **A reason is a claim too.** Measure it before you write it: narrow the
  item and build before you write that the compiler requires it.
- **A claim about an external system** states no more than its newest
  measurement shows.
- **A reference resolves from the repository.** A line holds no key of a
  tracker its reader may not open, no requirement id or term that only
  another repository defines, no word of the team's delivery process, and no
  person named as the owner of a decision: state the decision as the
  system's own, and link the spec or ADR that holds its reasoning. Before a
  branch lands, search the lines it adds for such keys, words and names,
  case-insensitive, and compare the hits with the main line. Why: the
  repository outlives the tracker and the team.
- **A fixed sentence has twins.** When a fix changes a sentence of a document
  or a comment, grep the tree for the same sentence and fix every twin in the
  same change. Why: a twin left behind keeps the defect the fix removed, and
  the next review finds it as a new one.

# Reviewing rather than restructuring

Asked only to review documents, run A1, name the two or three heaviest
sections with their shares, say where each belongs, and stop. Asked to review
comments, run B1 and report the densest files and the rot candidates that hold.
The table is the finding; a rewrite nobody asked for is not.
