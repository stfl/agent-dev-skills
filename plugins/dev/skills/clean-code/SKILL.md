---
name: clean-code
description: Language-agnostic design and cleanup discipline — state lives in the type, decompose by decision into pure single-purpose functions, deep modules over tidy fragments, and a second pass that leaves touched code cleaner without mixing structure and behaviour changes. Use when writing or refactoring code in any language, when deciding where a file or a directory goes, when a function grows conditionals over several fields, when asked to clean up, tidy, simplify, reduce technical debt, or leave code cleaner, and as the base every language skill (such as `rust`) inherits.
---

# Clean code

Complexity is incremental. No single change makes a codebase spaghetti; a hundred locally reasonable ones do. So the discipline has two halves: three design rules that keep a change from adding complexity, and a second pass that removes what slipped through. Debt is paid in that pass or ledgered as an issue. It is never silent.

## Three design rules

**1. State lives in the type.** When behaviour depends on the combination of several values, the combination is one sum type and the behaviour is one exhaustive match over it. The tells: a condition that tests two or more fields together; two booleans describing one thing; an optional field only meaningful beside another; a stored value recomputable from other fields. The move: one variant per legal combination, each carrying only the data that exists in that state, and derived values computed from the variant, never stored. Rust `enum`, a TypeScript discriminated union, a Kotlin sealed class, a Python `match` over dataclass variants — same rule, the language decides the spelling.

**2. Decompose by decision.** A function is a decision or an effect, not both. Decisions are pure: values in, values out, testable with literals and no mock. Effects sit in a thin shell that executes what the decision returned. What crosses that seam is the executable plan with its arguments filled in, not a label the shell must still interpret. Extract along decision boundaries, never by line count: a decision is the unit that is understandable and testable alone.

**3. Contracts describe; generics where they buy something.** An interface — trait, protocol, abstract base — names what every implementor must uphold, and every rule's inputs appear in its signature. Write one when a second implementation exists or a real seam does, and reach for the language's standard interfaces before a bespoke one. A generic parameter earns its place when it removes a runtime branch or a dynamic dispatch; otherwise the concrete type wins.

## Deep, then tidy

Two scales, one budget. Depth is measured at the interface: how much behaviour a caller gets per fact they must learn (`codebase-design` owns that vocabulary). Tidiness is measured inside the body: small functions, guard clauses, one idea per block. A deep module is usually built from many tidy private functions, and those cost the caller nothing. The failure is promoting tidy fragments to the interface: every public function is one more fact to learn, and a five-line function with five parameters is shallower than the twenty lines it replaced. Split when the piece is separable, which is to say a decision; keep together what shares state and is only ever read together. Beck's *one pile* is the sanctioned reverse move: inline scattered fragments, then re-split along the better seam.

## The tree is the table of contents

A directory holds one topic: a thing a reader looks for by name, whose files change together for one reason. Name it with the domain's noun for that topic, never with a kind of code: `utils/`, `types/`, `traits/`, `helpers/` and `common/` gather files that change for unrelated reasons and scatter every topic across them. Read from the top, the tree then lists what the program does, with the frame every topic uses — the entry point, configuration, logging — flat beside the topics. The test is deletion: dropping a feature drops one directory. A topic with one file stays a file, a directory holds another only when the inner group passes the same test, and a group usable on its own is a package of its own. It is *cohesion order* one level up; the language skill says what else a directory enforces. Why: a reader finds code by the name of what it does, and a change that stays inside one directory is reviewed whole.

## The second pass

Runs after the behaviour change is green, in a fresh context — a reviewer or a new agent — because the writer stops at green and inherits every "good enough" it told itself. Scope is the diff plus one hop: the callers and callees of every changed function, and every type that gained a field or variant. Nothing outside the scope moves.

1. **Two hats.** Structure changes and behaviour changes never share a commit. A tidying commit contains only tidyings.
2. **When.** Tidy *first* when it makes the coming change easier. Tidy *after* when the change taught you the shape. Tidy *later*, as a tracked issue, when it is outside the scope. Tidy *never* for code that will not change again. Duplication waits for the third copy.
3. **Safety.** Only under green tests, in steps small enough that a failure points at one move, tests after every step. Untested code gets a characterisation test first; when none can be got in, sprout new code into a new function rather than growing the old one.
4. **Worklist.** `references/smells.md`: one row per smell, with its detector and its move. The language skill supplies the mechanical detectors; for Rust that is clippy under `clippy.toml`.
5. **Done.** Every hit inside the scope is fixed or ledgered as an issue naming the smell and the location. No TODO comments. The report lists each candidate as Critical, High, Nice, or Skip with a reason, and closes with what was done.

Critical is a divergent copy of one business rule, or a control flow that hides a risky path: fix now. High is a function coordinating several responsibilities, a magic number, an unclear name: fix in this pass. Nice goes to the ledger. Skip is said in those words.

## Tidyings

The small moves, each safe alone, from Beck's *Tidy First?*: guard clause · delete dead code · normalize symmetries · new interface, old implementation · reading order · cohesion order · move declaration to first use · explaining variable · explaining constant · explicit parameters · chunk statements · extract helper · one pile · explaining comment · delete redundant comment.

## Not clean

- **Speculative generality.** An abstraction with one user, a parameter nobody passes, a generic with one instantiation. Delete it.
- **DRY is about knowledge, not text.** Two rules that share a body today are a coincidence. Merging them makes one edit silently change two rules. Extract only when the two would change together.
- **Extraction by line count.** See *Deep, then tidy*.
- **Comments doing the type's job.** A comment that says which fields are valid together is rule 1 unapplied. Keep comments for *why* and for invariants the signature cannot state.

## Language skills

A language skill inherits this file and adds only what the language changes: the construct that realises each rule, the mechanical detectors, and the traps its compiler does not catch. `rust` is one.
