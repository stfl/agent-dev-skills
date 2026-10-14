# agent-dev-skills

Engineering and documentation skills and rules for Claude Code, shared between
my own sessions, my Paperclip agents and teammates. Every skill is a plain
`SKILL.md` directory in the Agent Skills format, so Codex, opencode and
Paperclip read the same files.

| Plugin | Contents |
|--------|----------|
| `dev` | Engineering skills: clean code, Rust. A hook refuses edits to `.rs` files until the session has loaded the Rust skill |
| `writing` | Documentation skills, and the documentation style rule loaded into every session |

## Install

```
/plugin marketplace add stfl/agent-dev-skills
/plugin install dev@stfl-agent-dev-skills
/plugin install writing@stfl-agent-dev-skills
```

`/plugin marketplace update stfl-agent-dev-skills` pulls new versions.

## Edit while you use them

An installed plugin is a read-only snapshot. To change a skill as you use it,
clone the repository and link a plugin directory into `~/.claude/skills/`:
Claude Code loads any directory there that holds `.claude-plugin/plugin.json`
as a plugin, straight from your clone.

```
git clone git@github.com:stfl/agent-dev-skills ~/work/agent-dev-skills
ln -s ~/work/agent-dev-skills/plugins/dev ~/.claude/skills/dev
```

Do not also install that plugin from the marketplace: it would load twice.

## Contributing

`main` changes through pull requests. Open one from your clone or a fork.
