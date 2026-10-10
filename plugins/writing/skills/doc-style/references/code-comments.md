# Comments in code

Consulted from `doc-style` part B. Each rule carries its source; **[2°]** marks
a quote checked only through a secondary source, **[inferred]** a rule no
source states outright. Gathered 2026-10-08.

## What earns a line

The guides agree on *what* to write — why, and what the caller cannot see —
and disagree on *how much*: Ousterhout writes "5-10x more lines of comments"
than Martin, for whom "comments are always failures"
([their debate](https://github.com/johnousterhout/aposd-vs-clean-code)). This
skill sides with less, for one reason the debate does not settle: nothing
checks a comment, so every line is a liability until proven otherwise.

1. **Delete what restates the name, the types or the next line.** "There is no
   benefit of explicitly writing it into the documentation"
   ([rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html));
   "Do not state the obvious"
   ([Google C++](https://google.github.io/styleguide/cppguide.html#Comments));
   "Don't duplicate the function or class name at the beginning of the
   comment" ([LLVM](https://llvm.org/docs/CodingStandards.html#commenting)).
   Mechanical test: a doc comment must hold at least one word not already in
   the code
   ([jsdoc/informative-docs](https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/rules/informative-docs.md)).
2. **Keep the why, the constraint and the warning.** "Write comments that say
   *why* you have done a thing, not *what*"
   ([rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/conventions.html));
   "protect a well-meaning future editor from mistakenly 'fixing' your code"
   ([Google TotT](https://testing.googleblog.com/2017/07/code-health-to-comment-or-not-to-comment.html));
   "NEVER try to explain HOW your code works"
   ([Linux kernel](https://www.kernel.org/doc/html/latest/process/coding-style.html#commenting)).
3. **Change the code before commenting it** — an explaining variable, an
   extracted function, a better name, an assertion (TotT). "Often if code is so
   complicated that it needs lots of comments, then it could probably be
   simplified first"
   ([Chromium](https://chromium.googlesource.com/chromium/src/+/main/docs/documentation_guidelines.md)).
4. **A `///` is the caller's contract, not the implementation.** "Doc comments
   should not explain internal details such as the algorithm used"
   ([Go](https://go.dev/doc/comment)); rustdoc shows "everything users need to
   use the crate fully and nothing more"
   ([C-HIDDEN](https://rust-lang.github.io/api-guidelines/documentation.html#c-hidden)).
   In std every sentence of a doc comment is a stability promise
   ([std-dev-guide](https://std-dev-guide.rust-lang.org/breaking-changes/doc-changes.html)) —
   write each one as if it were.
5. **The first paragraph is one short sentence**
   ([RFC 1574](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)).
   Clippy's `too_long_first_doc_paragraph` draws the line at 200 characters.
6. **Leave a trait impl undocumented** unless it narrows or extends the trait's
   contract (Google C++, Google Python §3.8.3.1).
7. **The keep test.** Keep a comment only if a maintainer who studied the code
   carefully would still misread it without one
   ([OpenAI GPT-5 guide](https://github.com/openai/openai-cookbook/blob/main/examples/gpt-5/gpt-5_prompting_guide.ipynb)).
   Martin's version: "the best comments tell me something surprising and
   verifiable".

## What rots

Code and comment changed out of step are about 1.5 times as likely to lead to
a bug-introducing commit ([arXiv 2409.10781](https://arxiv.org/abs/2409.10781)).

1. **No second copy of a fact the code holds** — a constant's value, a count of
   variants, workers, tests or lints, a list of callers, a path, a line number.
   Link the owning item; `rustdoc::broken_intra_doc_links` warns when the link
   stops resolving ([rustdoc lints](https://doc.rust-lang.org/rustdoc/lints.html)).
   "Comments are easier to maintain if they are higher-level and more abstract
   than the code" (Ousterhout [2°]). No guide bans counts by name; the rule
   follows from single-source-of-truth [inferred].
2. **No callers, no context.** An exception's docstring says "what the
   exception represents, and not the context in which it might occur"
   ([Google Python §3.8.4](https://google.github.io/styleguide/pyguide.html)).
   PostgreSQL, 2017: "The comment above ParallelQueryMain() still refers to
   ParallelMain() as its caller which is no longer the case"
   ([thread](https://hackorum.dev/topics/35845)).
3. **No history.** "Comments describe present state, not past or transitions"
   ([writing-evergreen-comments](https://smithery.ai/skills/obra/writing-evergreen-comments)).
   The `doc-wording` rule carries the word list.
4. **Turn a claim into a check.** TotT replaces `// Safe since height is always
   > 0` with `checkArgument(height > 0)`; "if there is an invariant, document
   it, enforce it in the 'constructor' function"
   ([rust-analyzer](https://rust-analyzer.github.io/book/contributing/style.html)).
   A doctest is a comment the gate runs.
5. **The narrowest scope, and more abstract the farther away.** "The farther a
   comment is from the code it describes, the more abstract it should be"
   (Ousterhout [2°]). A module header that names a constant's value is too
   close to the code for its distance.

## No lint holds the line

No rustc, rustdoc or Clippy lint limits documentation volume or comment
density. The `missing_*` lints (`missing_docs`, `missing_errors_doc`,
`missing_panics_doc`, `missing_docs_in_private_items`) all push the other way.
`too_long_first_doc_paragraph` (nursery, exported items only, fixed at 200
characters) is the only length check, and `too_many_lines` skips comment
lines. Elsewhere, PMD's `CommentSize` is the one real limit found; SonarQube
deprecated its minimum-density rule in 2022 because "developers may be tempted
to write fake/useless comments"
([SonarSource](https://community.sonarsource.com/t/deprecation-of-common-server-rules-sonar-16051/71088)).
The discipline is by hand, which is why `scripts/comments.py` exists.

Agents over-comment by default, and the vendors' own prompts say so:
"Only add comments where the logic isn't self-evident"
([Anthropic](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices));
"Usage of these comments should be rare"
([Codex](https://github.com/openai/codex/blob/main/codex-rs/core/gpt_5_codex_prompt.md)).

## Where the reasoning goes instead

| Text | Home |
|---|---|
| What one item does for its caller | its `///` |
| What several items share; an invariant across a module | the module's `//!` |
| Why the design is this shape; what was rejected | the spec or ADR — small, frozen when superseded ([Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)) |
| The map of the codebase; deliberate absences | `ARCHITECTURE.md` or the agent notes — "Keep it short", "Only specify things that are unlikely to frequently change" ([matklad](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html)) |
| What changed and why | the commit message ([kernel](https://www.kernel.org/doc/html/latest/process/submitting-patches.html)) |
| What an operator sets | `docs/` |

Ousterhout's caveat [2°]: if a reader needs a fact to understand the *current*
code, it belongs in the code — which is the why, never the narrative.
