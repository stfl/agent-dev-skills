---
name: clean-code
description: Language-agnostic design and cleanup discipline — state lives in the type, decompose by decision into pure single-purpose functions, deep modules over tidy fragments, and a second pass that leaves touched code cleaner without mixing structure and behaviour changes. Use when writing or refactoring code in any language, when deciding where a file or directory goes, when a function grows conditionals over several fields, when asked to clean up, tidy, simplify, reduce technical debt, or leave code cleaner, and as the base every language skill (such as `rust`) inherits.
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

A directory holds one topic, named with the domain's noun (`billing/`, not `types/`): the files a reader looks for under that name, which change together for one reason. The frame every topic uses (the program's entry point, configuration, logging) stays flat at the root. A layout the framework fixes, such as Rails's `app/models/` or Django's `migrations/`, is frame too: the framework finds the files by it. The test is deletion: dropping a feature drops one directory. A one-file topic stays a file, a directory nests another only when the inner group passes the same test, and a group usable on its own becomes a package. It is *cohesion order* one level up. Why: a reader finds code by the name of what it does, and a change that stays inside one directory is reviewed whole.

## Reconcile against what you can read back

A design that brings a system to a desired state compares it with the actual state in one rule, stated in one place that every other place points to, which lists each compared key with where its actual value is read back; a key that cannot be read back decides nothing. On a third party's change notification the design reads the whole state again rather than applying the notification, since one user act can be several writes, each with its own notification, in any order. Before a design rests on what a third party's UI lets a user enter, or on what its server does on a write, read that form and its submit code and that server method, defaults included, at the version the project runs. Why: a third party's API, UI and notifications each show a different part of its behaviour, and only its code at that version shows which part the design meets.

## A test that tells right from wrong

A test covers each wrong form of the property it names. That one failing item takes nothing from the items around it can fail two ways, stopping at the failure or discarding what came before it, so its test puts the failing item between two that succeed and asserts the results of both; a failure at either end of the sequence shows one wrong form only. A test of a rule over several kinds of input gives one input of each kind. Before you rely on such a test, run it once against each wrong form and watch it fail. Why: a test that sees one wrong form passes code that is wrong in the other.

Code or a test that models what a tool does with an input — its output, its error text, its formatting, the place it reports — takes the expected value, the parser or the stand-in from the tool itself: run it on a real input of each kind the model covers and read what it prints, never write it from memory or from the tool's documentation. Why: a model written from expectation holds what its author believes, and the tool's output decides otherwise for the inputs that matter.

A test that a guard stops a costly input within a time bound sets the bound from the ratio of work between what the guard does before it stops and what the input costs without it: the bound tells a stop from an unguarded run on every machine where the stop takes less than the bound and more than the bound divided by that ratio. Set it at no less than five times the slowest CI run known then; afterwards a CI run is a finding only past half the bound, and only then is the bound set again. A measured time goes in the commit message or the pull request, with the run or host it came from, never in the docs or a comment. Why: a measured time is a sample of one machine, so the next slow runner makes a bound or a doc set from it false while the code stays the same.

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
