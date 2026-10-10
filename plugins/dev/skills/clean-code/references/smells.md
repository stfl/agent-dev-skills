# Smells and moves

The second pass's worklist. Names follow Fowler's *Refactoring* and Beck's *Tidy First?* so the literature stays searchable. A detector is what to grep for or what a lint flags; the language skill names the lint. The last column shows the shape in two languages; the rule holds in any.

| Smell | Detector | Move | Target shape |
|---|---|---|---|
| **Temporary field** | a field meaningful only in some states; an optional field valid only beside another; a `bool` pair | Replace Type Code with State | one sum type, each variant carrying only its own data — Rust `enum`, TS discriminated union |
| **Repeated switches** | the same discriminant matched in several functions | Replace Conditional with Polymorphism | closed set: one exhaustive `match` per operation, as a method on the type; open set: trait or interface |
| **Derived state stored** | a field recomputable from others; a cache with no invalidation rule | Replace Derived Variable with Query | a method; store only what cannot be recomputed |
| **Flag argument** | a `bool` parameter; a call site reading `f(x, true)` | Remove Flag Argument | an enum parameter, or two functions |
| **Primitive obsession** | an id, unit, or bounded value as `u64` or `string` | Replace Primitive with Object | newtype, branded type, or value object, parsed once at the boundary |
| **Data clumps** | the same three parameters travelling together | Introduce Parameter Object | a struct; a builder past four optional fields |
| **Decision and effect in one body** | IO — an `await`, a `?` on IO, a write — in the same function as the branching | Split Phase; Separate Query from Modifier | a pure decision returning a plan; a shell executing it |
| **Nested conditionals** | three levels of `if`; an `else` after a return | Replace Nested Conditional with Guard Clauses; Decompose Conditional | early return, `let else`, named predicates |
| **Long function** | past the size threshold *and* more than one decision inside | Extract Function, along the decision | one function per decision; a single long decision stays whole |
| **Long parameter list** | past the argument threshold | Introduce Parameter Object; Preserve Whole Object | a struct; pass the object, not five of its fields |
| **Duplicated knowledge** | the same rule in two places — not the same text | Extract Function | one function; two look-alike rules stay separate |
| **Speculative generality** | one implementor; a generic with one instantiation; an unused parameter | Inline; Remove Dead Code | the concrete thing |
| **Divergent change** | one module edited for unrelated reasons | Extract Module; Move Function | a module owns one invariant |
| **Shotgun surgery** | one reason touches many modules | Move Function; Inline Module | the knowledge in one place |
| **Feature envy, message chains** | a function reading another type's fields more than its own; `a.b().c().d()` | Move Function; Hide Delegate | the behaviour lives with the data |
| **Mutable shared state** | a value mutated from several places | own it in one place; pass values; return new values | one owner; a channel or a return value instead of shared mutation |
| **Loop doing two things** | an accumulator plus a side effect in one loop | Split Loop; Replace Loop with Pipeline | one iterator chain per concern |
| **Comment explaining what** | a comment restating the next line; a comment listing the valid field combinations | Rename; Extract; delete; or apply *Temporary field* | comments for *why* and for invariants the signature cannot state |

Two rows run in the tidy-to-deep direction and belong in the same pass: *Speculative generality*, and the "a single long decision stays whole" clause of *Long function*. A pass that only splits produces shallow modules.
