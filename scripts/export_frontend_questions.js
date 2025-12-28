#!/usr/bin/env node
/**
 * Extracts the hardcoded questions from the frontend Svelte page and writes them
 * as backend-ready JSON fixtures under data/questions.json.
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const frontendPath = path.resolve(__dirname, "../../verbumdei-web/src/routes/+page.svelte");
const outputPath = path.resolve(__dirname, "../data/questions.json");

const file = fs.readFileSync(frontendPath, "utf8");
const match = file.match(/const questions\s*:\s*Question\[\]\s*=\s*(\[[\s\S]*?\]);/);

if (!match) {
  console.error("Could not find questions array in +page.svelte");
  process.exit(1);
}

const arrayLiteral = match[1];
const questions = vm.runInNewContext("(" + arrayLiteral + ")");

const normalized = questions.map((q, idx) => {
  const options = q.options.map((o) => ({
    text: o.text,
    correct: o.correct,
    explanation: o.explanation,
  }));

  const base = {
    stage: idx + 1,
    stage_label: q.stage,
    prompt: q.prompt,
    options,
    tags: [],
  };

  if (q.source) {
    base.source = q.source;
  }

  return base;
});

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, JSON.stringify(normalized, null, 2));
console.log(`Wrote ${normalized.length} questions to ${outputPath}`);
