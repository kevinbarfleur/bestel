#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const agentDir = path.join(root, '.claude', 'agents');
const skillDir = path.join(root, '.claude', 'skills');
const commandDir = path.join(root, '.claude', 'commands');

let failed = false;
function fail(msg) {
  failed = true;
  console.error(`[FAIL] ${msg}`);
}
function ok(msg) {
  console.log(`[OK] ${msg}`);
}
function read(p) {
  return fs.readFileSync(p, 'utf8');
}
function walk(dir, predicate = () => true) {
  if (!fs.existsSync(dir)) return [];
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(p, predicate));
    else if (predicate(p)) out.push(p);
  }
  return out;
}
function parseYamlFrontmatter(text) {
  if (!text.startsWith('---\n')) return null;
  const end = text.indexOf('\n---', 4);
  if (end < 0) return null;
  const raw = text.slice(4, end).trim();
  const fields = new Map();
  for (const line of raw.split(/\r?\n/)) {
    const idx = line.indexOf(':');
    if (idx < 0) continue;
    fields.set(line.slice(0, idx).trim(), line.slice(idx + 1).trim());
  }
  return fields;
}

if (!fs.existsSync(path.join(root, 'CLAUDE.md'))) fail('CLAUDE.md missing');
else ok('CLAUDE.md exists');

const agentFiles = walk(agentDir, p => p.endsWith('.md'));
if (agentFiles.length === 0) fail('no .claude/agents/*.md files found');
const agentNames = new Set();
for (const file of agentFiles) {
  const rel = path.relative(root, file);
  const fm = parseYamlFrontmatter(read(file));
  if (!fm) {
    fail(`${rel}: missing YAML frontmatter`);
    continue;
  }
  const name = fm.get('name');
  const description = fm.get('description');
  if (!name) fail(`${rel}: missing name`);
  if (!description) fail(`${rel}: missing description`);
  if (name && agentNames.has(name)) fail(`${rel}: duplicate agent name ${name}`);
  if (name) agentNames.add(name);
  if (description && description.length < 80) fail(`${rel}: description too short for reliable routing`);
}
ok(`${agentFiles.length} agent files checked`);

const skillFiles = walk(skillDir, p => path.basename(p) === 'SKILL.md');
if (skillFiles.length === 0) fail('no .claude/skills/*/SKILL.md files found');
for (const file of skillFiles) {
  const rel = path.relative(root, file);
  const fm = parseYamlFrontmatter(read(file));
  if (!fm) {
    fail(`${rel}: missing YAML frontmatter`);
    continue;
  }
  if (!fm.get('name')) fail(`${rel}: missing name`);
  if (!fm.get('description')) fail(`${rel}: missing description`);
}
ok(`${skillFiles.length} skill files checked`);

const commandFiles = walk(commandDir, p => p.endsWith('.md'));
if (commandFiles.length === 0) fail('no .claude/commands/*.md files found');
else ok(`${commandFiles.length} command files checked`);

const textFiles = [
  path.join(root, 'CLAUDE.md'),
  path.join(root, 'AGENTS.md'),
  ...agentFiles,
  ...skillFiles,
  ...commandFiles,
  ...walk(path.join(root, 'docs', 'agentic'), p => p.endsWith('.md')),
].filter(fs.existsSync);

const forbidden = [
  { needle: 'wiki_open', reason: 'obsolete/nonexistent Bestel tool name; use wiki_parse' },
  { needle: 'Claude CLI, Codex CLI', reason: 'outdated provider wording for current Bestel architecture' },
];
for (const file of textFiles) {
  const body = read(file);
  for (const rule of forbidden) {
    if (body.includes(rule.needle)) fail(`${path.relative(root, file)}: contains ${JSON.stringify(rule.needle)} (${rule.reason})`);
  }
}
ok(`${textFiles.length} config text files scanned for obvious drift`);

if (failed) process.exit(1);
console.log('agentic config lint passed');
