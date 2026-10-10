# The human-facing README

Consulted from `doc-style` step A5. Sourced from the guides at the bottom; where
they disagree, the disagreement is stated rather than averaged away.

## Section order

Near-unanimous across Make a README, the Standard Readme spec and GitHub's own
guidance. Skip what the project has no answer for; keep the order.

| # | Section | Job | Length |
|---|---|---|---|
| 1 | Title + tagline | What this is, under 15 words. Standard Readme caps the description at **120 characters** | 1 line |
| 2 | Badges | Credibility, not decoration. **≤5**, and only ones currently green | 1 line |
| 3 | Hook | The problem, in the reader's terms, before the solution | 2–4 sentences |
| 4 | What it is + why | What it does, for whom, how it differs from the obvious alternative | 80–150 words |
| 5 | Proof it works | A GIF, screenshot, asciinema, or one command with its real output | 1 visual |
| 6 | Highlights | Scannable differentiators. Bullets only, never paragraphs | 4–6 bullets |
| 7 | Quickstart | A working result in under five minutes, copy-pasteable | code + <100 words |
| 8 | Install | Every supported path, with the platform/version each covers | code + short prose |
| 9 | Limits | What it does not do | 4–6 bullets |
| 10 | Where to go next | A table routing to `docs/`, architecture, examples | table |
| 11 | Support + contributing | Where to ask, whether PRs are accepted | 2 lines |
| 12 | License | SPDX name + link to `LICENSE`. **Must be last** | 1 line |

Quickstart before Install is a legitimate inversion: the three commands show the
shape and earn the install. Do not invert further — install below the fold, after
benchmarks or history, is a named anti-pattern.

## Above the fold

The first screenful decides whether the reader stays. It must deliver, in order:
what this is in one sentence; proof it works; the one command that produces a
first result — ideally inside the first ~200 words. The reader judges fit in
about 60 seconds without scrolling.

GitHub frames a README as answering five questions near the top: what it does,
why it is useful, how to get started, where to get help, who maintains it.
"Where to get help" is the one most often missing.

## Length

Multiple guides converge on **800–1,200 words** as the point where a README
starts shedding content into linked docs.

Make a README dissents — *"too long is better than too short"* — but its next
sentence is *"consider separate documentation for excessive detail."* The advice
targets abandoned stubs, not a 7,000-word manual. Apply the threshold.

Standard Readme requires a table of contents over 100 lines. GitHub now
auto-generates one from the heading outline, so a hand-written TOC on a primer
mostly pushes the hook below the fold. A "Where to go next" table does the
routing better. Skip the TOC, keep the table.

## Prose vs structure

- One prose paragraph describes the product. After that, prose is the exception
  (Bugayenko: *"use as few words as you can"*).
- Break to headers, bullets, code or a table once a paragraph would run past
  4–6 lines.
- A table beats prose whenever you list more than three comparable items with
  two or more attributes each — platform × command, key × meaning, mode × effect.
- Never use `####`. If a README needs four heading levels, it is a reference
  document that has not admitted it yet.
- Show output, not only input. *"A few lines of syntax-highlighted source are
  worth a thousand words"* (thoughtbot).
- Use relative links. GitHub rewrites them per branch and they survive a clone.

## Selling without a brochure

Every strong systems README sells by **demonstration, not adjective**. No
"blazing fast", no "enterprise-grade", no "production-ready" — a benchmark
table, a real command, a real diff. Precision and candour read as confidence;
superlatives read as compensation.

| Exemplar | Why it works |
|---|---|
| [ripgrep](https://github.com/BurntSushi/ripgrep) | Plain one-sentence definition, real benchmark tables against grep/ag, and an explicit **"Why shouldn't I use ripgrep?"**. The honesty is the marketing |
| [bat](https://github.com/sharkdp/bat) | One-line pitch, then integration examples with fzf/find/ripgrep *before* install — sells by fitting the reader's existing workflow |
| [fd](https://github.com/sharkdp/fd) | Leads with what it replaces (`find`), then pipe-into-xargs examples — sells on pipeline fit, not feature count |
| [httpie](https://github.com/httpie/httpie) | Demo GIF immediately under the pitch, before any prose — visual proof replaces description |
| [esbuild](https://github.com/evanw/esbuild) | One differentiator ("why is this fast") with a graphic, instead of a feature wall |

## Anti-patterns

- **Wall of text** — no headers, code blocks or visuals. The most-cited failure.
- **Everything in one file** — API reference, architecture, changelog and
  contributor guide inline instead of linked. The failure this skill exists for.
- **Install buried below the fold**, after benchmarks, philosophy or history.
- **Unsupported superlatives** with no benchmark or demo behind them.
- **No stated limitations** — reduces trust with technical readers.
- **Stale support claims** — a badge or matrix listing a platform CI does not
  build. Verify against the CI and release matrices, not against the last badge.
- **Missing why and status** — the Prana study's empirical finding: *what* and
  *how* dominate READMEs while *why* and maintenance status are most often
  absent. Cheap to add, and rare enough to differentiate.
- **Absolute links** that break on clone.
- **Changelog, license text, or contributor lists inline** — the Releases tab,
  `LICENSE`, and GitHub's contributors graph already carry these.

## What leaves the README

| Cut | Goes to |
|---|---|
| Full API reference | Generated docs (rustdoc, docs.rs) or a `docs/` page |
| Every configuration option | `docs/configuration.md` |
| Architecture, thread models, internal seams | `ARCHITECTURE.md` or agent notes |
| Build, CI, packaging mechanics | Build docs or skills |
| Long troubleshooting or FAQ | `docs/troubleshooting.md` |
| Changelog | Releases tab / `CHANGELOG.md` |
| License text | `LICENSE` — the README names it |
| Design history, philosophy, dev journal | ADRs or a blog post |

## Sources

- [Make a README](https://www.makeareadme.com/) — the most-copied structural baseline.
- [Standard Readme spec](https://github.com/RichardLitt/standard-readme/blob/main/spec.md) — formal section contract; license last, 120-char description, TOC rule.
- [GitHub: About READMEs](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes) — the five questions, relative links, 500 KiB render truncation.
- [Diátaxis](https://diataxis.fr/) — tutorial/how-to/reference/explanation. A README is explanation plus a taste of tutorial; it hands off the other two.
- [Prana et al., *Categorizing the Content of GitHub README Files*](https://arxiv.org/abs/1802.06997) — 4,226 sections across 393 repos; why/status are the neglected categories.
- [Bugayenko, *Elegant READMEs*](https://www.yegor256.com/2019/04/23/elegant-readme.html) — one description paragraph, no `####`, omit what GitHub already surfaces.
- [thoughtbot, *How to write a great README*](https://thoughtbot.com/blog/how-to-write-a-great-readme) — code samples over prose; badges only if green.
- [awesome-readme](https://github.com/matiassingers/awesome-readme) — curated exemplars and the classic essays.
