import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import type { IkihsResult } from "./types.js";

const CLI_BIN = resolve(import.meta.dirname, "../../../target/debug/ikihs-cli");

export function highlightIkihs(source: string, lang: string, themePath: string): IkihsResult {
  if (!existsSync(CLI_BIN)) {
    throw new Error(`ikihs CLI not found at ${CLI_BIN}. Run 'cargo build -p ikihs-cli' first.`);
  }

  const themeArg = existsSync(themePath)
    ? themePath
    : resolve(import.meta.dirname, "../themes", themePath);

  const stdout = execSync(`"${CLI_BIN}" highlight -l "${lang}" -t "${themeArg}"`, {
    input: source,
    encoding: "utf-8",
    maxBuffer: 50 * 1024 * 1024,
  });

  return JSON.parse(stdout) as IkihsResult;
}
