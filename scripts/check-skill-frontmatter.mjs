#!/usr/bin/env node
// Parse every SKILL.md frontmatter as YAML and check it carries a name and a
// description. `claude plugin validate` reads the manifests, not the skills,
// so an unquoted `: ` inside a description reaches main without it.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { load } from "js-yaml";

function skillFiles(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return skillFiles(path);
    return entry === "SKILL.md" ? [path] : [];
  });
}

function problems(path) {
  const text = readFileSync(path, "utf8");
  const match = /^---\r?\n([\s\S]*?)\r?\n---\r?\n/.exec(text);
  if (!match) return ["no `---` frontmatter block at the top of the file"];

  let frontmatter;
  try {
    frontmatter = load(match[1]);
  } catch (error) {
    return [`frontmatter is not valid YAML — ${error.message.split("\n")[0]}`];
  }
  if (frontmatter === null || typeof frontmatter !== "object" || Array.isArray(frontmatter)) {
    return ["frontmatter does not parse to a mapping"];
  }

  return ["name", "description"].flatMap((key) => {
    const value = frontmatter[key];
    if (value === undefined) return [`\`${key}\` is missing`];
    if (typeof value !== "string" || value.trim() === "") return [`\`${key}\` is not a non-empty string`];
    return [];
  });
}

const failures = skillFiles("plugins").sort().flatMap((path) => {
  const found = problems(path);
  found.forEach((problem) => console.error(`✘ ${path}: ${problem}`));
  if (found.length === 0) console.log(`✔ ${path}`);
  return found;
});

if (failures.length > 0) {
  console.error(`\n${failures.length} problem(s). A description holding a \`: \` needs quotes around it.`);
  process.exit(1);
}
