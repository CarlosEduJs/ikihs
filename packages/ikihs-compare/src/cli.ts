import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { highlightShiki } from "./shiki.js";
import { highlightIkihs } from "./ikihs.js";
import { compareResults } from "./diff.js";
import { generateHtml } from "./html.js";

const [sourceFile, lang = "rust", themeName = "dark-plus", outFile] = process.argv.slice(2);

if (!sourceFile || !lang) {
  console.error(`
Usage:   pnpm compare <file> <lang> [theme] [output.html]

Example: pnpm compare sample.rs rust dark-plus
         pnpm compare sample.js js light-plus diff.html
`);
  process.exit(1);
}

const source = readFileSync(resolve(sourceFile), "utf-8").trimEnd();
const themePath = resolve(import.meta.dirname, `../themes/${themeName}.json`);

console.error(`Shiki: highlighting ${sourceFile} (${lang}, ${themeName})...`);
const shiki = await highlightShiki(source, lang, themeName);

console.error(`Ikihs: highlighting ${sourceFile} (${lang}, ${themeName})...`);
const ikihs = highlightIkihs(source, lang, themePath);

console.error(`Comparing...`);
const diffs = compareResults(ikihs, shiki);

// Print terminal summary
console.log(`\n── Comparison: ${sourceFile} ──`);
console.log(`Language: ${ikihs.language}`);
console.log(`Match score: ${diffs.summary.score}%`);
console.log(`  Exact:        ${diffs.summary.exact}`);
console.log(`  Color diff:   ${diffs.summary.colorDiff}`);
console.log(`  Offset diff:  ${diffs.summary.offsetDiff}`);
console.log(`  Extra (Ikihs): ${diffs.summary.extra}`);
console.log(`  Missing Ikihs: ${diffs.summary.missingIkihs}`);
console.log(`  Missing Shiki: ${diffs.summary.missingShiki}`);
console.log(`  Total tokens: ${diffs.summary.total}`);

// Print per-line diff
for (let li = 0; li < diffs.lines.length; li++) {
  const line = diffs.lines[li]!;
  const mismatches = line.filter((t) => t.match !== "exact");
  if (mismatches.length > 0) {
    console.log(`\n  Line ${li + 1}: ${mismatches.length} diff(s)`);
    for (const t of mismatches) {
      const pos = `[${t.offset}..${t.offset + t.content.length}]`;
      const content = JSON.stringify(t.content);
      console.log(
        `    ${t.match.padEnd(18)} ${pos.padEnd(12)} ${content.padEnd(20)} shiki=${t.shikiColor.padEnd(8)} ikihs=${t.ikihsColor.padEnd(8)} cat=${t.ikihsCategory}`,
      );
    }
  }
}

// Generate HTML
if (outFile) {
  const html = generateHtml(source, ikihs, shiki, diffs, themeName);
  const outPath = resolve(outFile);
  writeFileSync(outPath, html);
  console.log(`\nHTML report: ${outPath}`);
}
