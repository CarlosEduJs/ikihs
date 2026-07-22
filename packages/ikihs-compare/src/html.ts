import type { DiffToken, IkihsResult, ShikiResult } from "./types.js";

function colorClass(match: DiffToken["match"]): string {
  switch (match) {
    case "exact":
      return "";
    case "color_diff":
      return "diff-color";
    case "offset_diff":
      return "diff-offset";
    case "extra":
      return "diff-extra";
    case "missing_in_ikihs":
      return "diff-missing";
    case "missing_in_shiki":
      return "diff-missing-alt";
  }
}

function tokenHtml(
  content: string,
  color: string | undefined,
  extraClass: string,
  title: string,
): string {
  const style =
    color && extraClass !== "diff-missing" ? `style="color:${color}"` : `style="color:#666"`;
  return `<span class="token ${extraClass}" ${style} title="${title}">${esc(content)}</span>`;
}

function lineHtml(tokens: DiffToken[], side: "left" | "right"): string {
  return tokens
    .map((t) => {
      const color = side === "left" ? t.shikiColor : t.ikihsColor;
      const cls = colorClass(t.match);
      const title = `shiki:${t.shikiColor} ikihs:${t.ikihsColor} cat:${t.ikihsCategory}`;
      return tokenHtml(t.content, color, cls, title);
    })
    .join("");
}

function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/ /g, "&#160;")
    .replace(/\n/g, "<br>");
}

export function generateHtml(
  source: string,
  ikihs: IkihsResult,
  _shiki: ShikiResult,
  diffs: {
    lines: DiffToken[][];
    summary: ReturnType<typeof import("./diff.js").compareResults>["summary"];
  },
  themeName: string,
): string {
  const { lines, summary } = diffs;

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Ikihs vs Shiki — ${themeName}</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace; font-size: 13px; background: #1e1e1e; color: #ccc; padding: 20px; }
.header { margin-bottom: 16px; }
.header h1 { font-size: 18px; color: #fff; }
.summary { display: flex; gap: 24px; margin: 12px 0; }
.summary-item { background: #2d2d2d; padding: 8px 14px; border-radius: 6px; }
.summary-item .label { font-size: 11px; color: #888; }
.summary-item .value { font-size: 22px; font-weight: bold; color: #fff; }
.summary-item .value.good { color: #4ec9b0; }
.summary-item .value.warn { color: #dcdcaa; }
.summary-item .value.bad { color: #f44747; }
.columns { display: flex; gap: 2px; background: #2d2d2d; border-radius: 8px; overflow: hidden; }
.column { flex: 1; overflow-x: auto; padding: 12px; }
.column h3 { font-size: 12px; color: #888; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 1px; }
.column-shiki { background: #1e1e1e; }
.column-ikihs { background: #1e1e1e; }
.code { white-space: pre; line-height: 1.6; }
.line { display: flex; }
.line-number { color: #555; width: 36px; text-align: right; padding-right: 12px; user-select: none; }
.token { white-space: pre; }
.diff-color { border-bottom: 2px solid #dcdcaa; }
.diff-offset { border-bottom: 2px solid #569cd6; }
.diff-extra { background: rgba(86, 156, 214, 0.15); border-bottom: 2px solid #569cd6; }
.diff-missing { background: rgba(244, 71, 71, 0.15); border-bottom: 2px solid #f44747; }
.diff-missing-alt { background: rgba(244, 71, 71, 0.15); border-bottom: 2px solid #f44747; }
.legend { display: flex; gap: 16px; margin: 12px 0; flex-wrap: wrap; }
.legend-item { display: flex; align-items: center; gap: 4px; font-size: 11px; color: #999; }
.legend-swatch { width: 20px; height: 4px; border-radius: 2px; }
.legend-swatch.color { background: #dcdcaa; }
.legend-swatch.offset { background: #569cd6; }
.legend-swatch.missing { background: #f44747; }
</style>
</head>
<body>
<div class="header">
  <h1>Ikihs vs Shiki — ${themeName}</h1>
  <p style="color:#888;margin-top:4px">source: ${esc(source.slice(0, 80))}${source.length > 80 ? "…" : ""} | lang: ${ikihs.language}</p>
</div>

<div class="summary">
  <div class="summary-item">
    <div class="label">Match Score</div>
    <div class="value ${summary.score >= 90 ? "good" : summary.score >= 70 ? "warn" : "bad"}">${summary.score}%</div>
  </div>
  <div class="summary-item">
    <div class="label">Exact</div>
    <div class="value good">${summary.exact}</div>
  </div>
  <div class="summary-item">
    <div class="label">Color Diff</div>
    <div class="value warn">${summary.colorDiff}</div>
  </div>
  <div class="summary-item">
    <div class="label">Offset Diff</div>
    <div class="value warn">${summary.offsetDiff}</div>
  </div>
  <div class="summary-item">
    <div class="label">Extra in Ikihs</div>
    <div class="value bad">${summary.extra}</div>
  </div>
  <div class="summary-item">
    <div class="label">Missing in Ikihs</div>
    <div class="value bad">${summary.missingIkihs}</div>
  </div>
</div>

<div class="legend">
  <div class="legend-item"><div class="legend-swatch color"></div> Color mismatch</div>
  <div class="legend-item"><div class="legend-swatch offset"></div> Token boundary diff</div>
  <div class="legend-item"><div class="legend-swatch missing"></div> Missing token</div>
</div>

<div class="columns">
  <div class="column column-shiki">
    <h3>Shiki (reference)</h3>
    <div class="code">
${lines.map((l, i) => `      <div class="line"><span class="line-number">${i + 1}</span>${lineHtml(l, "left")}</div>`).join("\n")}
    </div>
  </div>
  <div class="column column-ikihs">
    <h3>Ikihs</h3>
    <div class="code">
${lines.map((l, i) => `      <div class="line"><span class="line-number">${i + 1}</span>${lineHtml(l, "right")}</div>`).join("\n")}
    </div>
  </div>
</div>
</body>
</html>`;
}
