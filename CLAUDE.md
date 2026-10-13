# agent-dev-skills — agent reference

A Claude Code plugin marketplace. `README.md` is the human primer.

## Layout

- `.claude-plugin/marketplace.json` lists the plugins, each by its directory
  under `plugins/`.
- `plugins/<plugin>/.claude-plugin/plugin.json` is a plugin's manifest. Claude
  Code discovers `skills/` and `hooks/hooks.json` inside the plugin directory
  on its own. A path a manifest declares must stay inside that plugin
  directory: Claude Code rejects one that resolves outside it.
- `plugins/<plugin>/skills/<name>/SKILL.md` is a skill. Keep the Agent Skills
  format, frontmatter `name` and `description`, so other agents read the same
  files. The description is what decides when a skill loads; put its triggers
  there.
- `plugins/writing/rules/*.md` are always-on rules. Plugins have no rules
  component, so `plugins/writing/hooks/rules.sh` prints every file there at
  session start. A plugin that needs its own rules copies that hook.

## Who consumes this

- Installs from the marketplace are read-only snapshots.
- An editor links a plugin directory of their own clone into
  `~/.claude/skills/` and edits there.
- Stefan's Paperclip agents take the `dev` skills from here as Paperclip
  company skills. Stefan reviews any change they propose before it reaches
  `main`.

## Workflow

- `main` changes only through pull requests.
- A change written by an agent waits for a human review, always. These files
  are instructions that sessions with root access follow; an unreviewed skill
  is a way around every sandbox those sessions run.
- Before opening a pull request, run what CI runs:
  `claude plugin validate --strict .` and the same for each plugin directory.
