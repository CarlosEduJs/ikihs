import type { IkihsResult, ShikiResult, DiffToken } from "./types.js";

function normalizeColor(c: string): string {
  return c.toLowerCase();
}

export function compareResults(
  ikihs: IkihsResult,
  shiki: ShikiResult,
): { lines: DiffToken[][]; summary: ReturnType<typeof summarize> } {
  const maxLines = Math.max(ikihs.tokens.length, shiki.tokens.length);
  const allLines: DiffToken[][] = [];

  for (let li = 0; li < maxLines; li++) {
    const ikihsLine = ikihs.tokens[li] ?? [];
    const shikiLine = shiki.tokens[li] ?? [];
    const lineDiffs = compareLine(ikihsLine, shikiLine);
    allLines.push(lineDiffs);
  }

  const summary = summarize(allLines);
  return { lines: allLines, summary };
}

function compareLine(
  ikihsTokens: IkihsResult["tokens"][number],
  shikiTokens: ShikiResult["tokens"][number],
): DiffToken[] {
  const result: DiffToken[] = [];
  let i = 0;
  let s = 0;

  while (i < ikihsTokens.length || s < shikiTokens.length) {
    if (i >= ikihsTokens.length) {
      result.push({
        content: shikiTokens[s]!.content,
        offset: shikiTokens[s]!.offset,
        ikihsColor: "",
        shikiColor: normalizeColor(shikiTokens[s]!.color),
        ikihsCategory: "",
        match: "missing_in_ikihs",
      });
      s++;
    } else if (s >= shikiTokens.length) {
      result.push({
        content: ikihsTokens[i]!.content,
        offset: ikihsTokens[i]!.offset,
        ikihsColor: normalizeColor(ikihsTokens[i]!.color),
        shikiColor: "",
        ikihsCategory: ikihsTokens[i]!.category,
        match: "missing_in_shiki",
      });
      i++;
    } else if (ikihsTokens[i]!.offset === shikiTokens[s]!.offset) {
      const iTok = ikihsTokens[i]!;
      const sTok = shikiTokens[s]!;
      const iColor = normalizeColor(iTok.color);
      const sColor = normalizeColor(sTok.color);
      const iEnd = iTok.offset + iTok.content.length;
      const sEnd = sTok.offset + sTok.content.length;

      if (iEnd === sEnd && iTok.content === sTok.content) {
        const match = iColor === sColor ? "exact" : "color_diff";
        result.push({
          content: iTok.content,
          offset: iTok.offset,
          ikihsColor: iColor,
          shikiColor: sColor,
          ikihsCategory: iTok.category,
          match,
        });
        i++;
        s++;
      } else if (iEnd <= sEnd) {
        result.push({
          content: iTok.content,
          offset: iTok.offset,
          ikihsColor: iColor,
          shikiColor: sColor,
          ikihsCategory: iTok.category,
          match: "offset_diff",
        });
        i++;
      } else {
        result.push({
          content: sTok.content,
          offset: sTok.offset,
          ikihsColor: iColor,
          shikiColor: sColor,
          ikihsCategory: iTok.category,
          match: "offset_diff",
        });
        s++;
      }
    } else if (ikihsTokens[i]!.offset < shikiTokens[s]!.offset) {
      result.push({
        content: ikihsTokens[i]!.content,
        offset: ikihsTokens[i]!.offset,
        ikihsColor: normalizeColor(ikihsTokens[i]!.color),
        shikiColor: "",
        ikihsCategory: ikihsTokens[i]!.category,
        match: "extra",
      });
      i++;
    } else {
      result.push({
        content: shikiTokens[s]!.content,
        offset: shikiTokens[s]!.offset,
        ikihsColor: "",
        shikiColor: normalizeColor(shikiTokens[s]!.color),
        ikihsCategory: "",
        match: "missing_in_ikihs",
      });
      s++;
    }
  }

  return result;
}

function summarize(lines: DiffToken[][]) {
  let total = 0;
  let exact = 0;
  let colorDiff = 0;
  let offsetDiff = 0;
  let extra = 0;
  let missingIkihs = 0;
  let missingShiki = 0;

  for (const line of lines) {
    for (const t of line) {
      total++;
      switch (t.match) {
        case "exact":
          exact++;
          break;
        case "color_diff":
          colorDiff++;
          break;
        case "offset_diff":
          offsetDiff++;
          break;
        case "extra":
          extra++;
          break;
        case "missing_in_ikihs":
          missingIkihs++;
          break;
        case "missing_in_shiki":
          missingShiki++;
          break;
      }
    }
  }

  const score = total > 0 ? Math.round((exact / total) * 100) : 100;
  return { total, exact, colorDiff, offsetDiff, extra, missingIkihs, missingShiki, score };
}
