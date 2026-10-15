#!/usr/bin/env node
// Check every SKILL.md frontmatter against the Agent Skills rules a skill is
// loaded by: it parses as YAML, and its `name` and `description` are there,
// inside their length limits, with a `name` that matches its own directory.
// `claude plugin validate` reads the manifests, not the skills, so an unquoted
// `: ` inside a description reaches main without this.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { load } from "js-yaml";

// https://agentskills.io/specification. Both limits are inclusive.
const MAX_NAME_LENGTH = 64;
const MAX_DESCRIPTION_LENGTH = 1024;
// Lowercase alphanumerics in hyphen-joined groups, so no leading, trailing or
// doubled hyphen.
const NAME_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

// The limits count characters, which are code points, not UTF-16 units.
const characters = (value) => [...value].length;

function skillFiles(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return skillFiles(path);
    return entry === "SKILL.md" ? [path] : [];
  });
}

function presenceProblems(key, value) {
  if (value === undefined) return [`\`${key}\` is missing`];
  if (typeof value !== "string" || value.trim() === "") return [`\`${key}\` is not a non-empty string`];
  return [];
}

function lengthProblems(key, value, limit) {
  const length = characters(value);
  if (length > limit) return [`\`${key}\` is ${length} characters, over the ${limit}-character limit`];
  return [];
}

function nameProblems(name, directory) {
  const missing = presenceProblems("name", name);
  if (missing.length > 0) return missing;

  return [
    ...lengthProblems("name", name, MAX_NAME_LENGTH),
    ...(NAME_PATTERN.test(name)
      ? []
      : [`\`name\` is \`${name}\`, which is not lowercase letters, digits and single inner hyphens`]),
    ...(name === directory
      ? []
      : [`\`name\` is \`${name}\` but its directory is \`${directory}\`; the two must be equal`]),
  ];
}

function descriptionProblems(description) {
  const missing = presenceProblems("description", description);
  if (missing.length > 0) return missing;

  return lengthProblems("description", description, MAX_DESCRIPTION_LENGTH);
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

  return [
    ...nameProblems(frontmatter.name, basename(dirname(path))),
    ...descriptionProblems(frontmatter.description),
  ];
}

const failures = skillFiles("plugins").sort().flatMap((path) => {
  const found = problems(path);
  found.forEach((problem) => console.error(`✘ ${path}: ${problem}`));
  if (found.length === 0) console.log(`✔ ${path}`);
  return found;
});

if (failures.length > 0) {
  console.error(`\n${failures.length} problem(s). A description holding a \`: \` needs quotes around it, and a renamed skill directory needs its \`name\` renamed with it.`);
  process.exit(1);
}
