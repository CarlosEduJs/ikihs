import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
  unlinkSync,
} from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createInterface } from "node:readline";
import { stdin as input, stdout as output } from "node:process";
import { parse as parseToml, stringify as stringifyToml } from "smol-toml";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const CHANGES_DIR = join(ROOT, ".changes");
const CARGO_TOML = join(ROOT, "Cargo.toml");
const CHANGELOG = join(ROOT, "CHANGELOG.md");

interface ChangeEntry {
  bump: "patch" | "minor" | "major";
  packages: string[];
  body: string;
}

function parseFrontmatter(content: string): { fm: Record<string, unknown>; body: string } {
  const match = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
  if (!match || match.length < 3) throw new Error("Invalid frontmatter");
  const raw = match[1]!;
  const body = match[2]!;
  const fm: Record<string, unknown> = {};
  for (const line of raw.split("\n")) {
    const idx = line.indexOf(":");
    if (idx === -1) continue;
    const key = line.slice(0, idx).trim();
    const val = line.slice(idx + 1).trim();
    if (val.startsWith("[")) {
      fm[key] = val
        .slice(1, -1)
        .split(",")
        .map((s) => s.trim().replace(/['"]/g, ""))
        .filter(Boolean);
    } else {
      fm[key] = val.replace(/^['"]|['"]$/g, "");
    }
  }
  return { fm, body: body.trim() };
}

function readChanges(): { files: string[]; entries: ChangeEntry[] } {
  if (!existsSync(CHANGES_DIR)) return { files: [], entries: [] };
  const files = readdirSync(CHANGES_DIR)
    .filter((f) => f.endsWith(".md") && f !== ".gitkeep")
    .sort();
  const entries = files.map((f) => {
    const content = readFileSync(join(CHANGES_DIR, f), "utf-8");
    const { fm, body } = parseFrontmatter(content);
    return {
      bump: (fm.bump as "patch" | "minor" | "major") || "patch",
      packages: (fm.packages as string[]) || [],
      body,
    } as ChangeEntry;
  });
  return { files, entries };
}

function deleteChanges(files: string[]) {
  for (const f of files) unlinkSync(join(CHANGES_DIR, f));
}

function bumpSemver(v: string, b: "patch" | "minor" | "major"): string {
  const parts = v.split(".").map(Number);
  const major = parts[0] ?? 0;
  const minor = parts[1] ?? 0;
  const patch = parts[2] ?? 0;
  switch (b) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
  }
}

function bumpPriority(b: "patch" | "minor" | "major"): number {
  return b === "major" ? 2 : b === "minor" ? 1 : 0;
}

function dateStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function readCargoVersion(): string {
  const raw = readFileSync(CARGO_TOML, "utf-8");
  const doc = parseToml(raw) as { workspace?: { package?: { version?: string } } };
  const version = doc.workspace?.package?.version;
  if (!version) throw new Error("Could not read workspace version from Cargo.toml");
  return version;
}

function writeCargoVersion(version: string) {
  const raw = readFileSync(CARGO_TOML, "utf-8");
  const doc = parseToml(raw) as Record<string, unknown>;
  const wp = (doc as Record<string, Record<string, unknown>>).workspace?.package as
    | Record<string, unknown>
    | undefined;
  if (wp) wp.version = version;
  writeFileSync(CARGO_TOML, stringifyToml(doc));
  console.log(`  Cargo.toml → ${version}`);
}

function readNpmVersion(pkgJson: string): string {
  const raw = readFileSync(pkgJson, "utf-8");
  const pkg = JSON.parse(raw) as { version?: string };
  if (!pkg.version) throw new Error(`Could not read version from ${pkgJson}`);
  return pkg.version;
}

function writeNpmVersion(pkgJson: string, version: string) {
  const raw = readFileSync(pkgJson, "utf-8");
  const pkg = JSON.parse(raw) as Record<string, unknown>;
  pkg.version = version;
  writeFileSync(pkgJson, JSON.stringify(pkg, null, 2) + "\n");
  console.log(`  ${pkgJson} → ${version}`);
}

function buildChangelogEntries(entries: ChangeEntry[], version: string): string {
  const lines: string[] = [`## ${version} (${dateStr()})\n`];
  const groups: Record<string, string[]> = { Major: [], Minor: [], Patch: [] };
  for (const e of entries) {
    const group = e.bump === "major" ? "Major" : e.bump === "minor" ? "Minor" : "Patch";
    const pkg = e.packages.length ? ` (${e.packages.join(", ")})` : "";
    groups[group]!.push(`-${pkg} ${e.body}`);
  }
  for (const g of ["Major", "Minor", "Patch"] as const) {
    if (groups[g]!.length) {
      lines.push(`### ${g} Changes\n`);
      lines.push(...groups[g]!.map((l) => l + "\n"));
    }
  }
  return lines.join("\n");
}

function updateChangelog(newSection: string) {
  if (existsSync(CHANGELOG)) {
    const existing = readFileSync(CHANGELOG, "utf-8");
    const header = "# Changelog\n\n";
    writeFileSync(
      CHANGELOG,
      header + newSection + "\n" + existing.replace(/^# Changelog\n\n?/, ""),
    );
  } else {
    writeFileSync(CHANGELOG, `# Changelog\n\n${newSection}\n`);
  }
  console.log("  CHANGELOG.md updated");
}

// ─── Subcommands ───

async function cmdAdd() {
  const rl = createInterface({ input, output });
  const ask = (q: string): Promise<string> => new Promise((r) => rl.question(q, r));

  console.log("Creating changelog entry...\n");
  const bumpRaw = (await ask("Bump type (patch|minor|major) [patch]: ")).trim() || "patch";
  const bump = ["patch", "minor", "major"].includes(bumpRaw)
    ? (bumpRaw as "patch" | "minor" | "major")
    : "patch";
  const pkgRaw = (await ask("Packages (comma-separated, empty=all): ")).trim();
  const packages = pkgRaw
    ? pkgRaw
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean)
    : [];
  const body = (await ask("Changelog description: ")).trim();
  rl.close();

  if (!body) {
    console.error("error: description is required");
    process.exit(1);
  }

  if (!existsSync(CHANGES_DIR)) mkdirSync(CHANGES_DIR, { recursive: true });
  const ts = Date.now();
  const file = join(CHANGES_DIR, `${ts}.md`);
  const pkgYaml = packages.length
    ? `packages:\n${packages.map((p) => `  - ${p}`).join("\n")}`
    : `packages: []`;
  writeFileSync(file, `---\nbump: ${bump}\n${pkgYaml}\n---\n\n${body}\n`);
  console.log(`\nCreated: .changes/${ts}.md`);
}

function cmdVersion() {
  const { files, entries } = readChanges();
  if (!files.length) {
    console.log("No changelog entries found in .changes/");
    return;
  }

  const maxBump = entries.reduce(
    (max, e) => (bumpPriority(e.bump) > bumpPriority(max) ? e.bump : max),
    "patch" as "patch" | "minor" | "major",
  );

  const allPkgs = entries.some((e) => !e.packages.length);
  const affectedNpmPkgs: string[] = allPkgs
    ? ["packages/ikihsjs/package.json", "packages/ikihs-compare/package.json"]
    : [...new Set(entries.flatMap((e) => e.packages))].flatMap((name) => {
        if (name === "ikihsjs") return ["packages/ikihsjs/package.json"];
        if (name === "@ikihs/compare") return ["packages/ikihs-compare/package.json"];
        return [];
      });

  const cargoVersion = readCargoVersion();
  const newCargoVersion = bumpSemver(cargoVersion, maxBump);
  writeCargoVersion(newCargoVersion);

  for (const pkgJson of affectedNpmPkgs) {
    const fullPath = join(ROOT, pkgJson);
    const oldVer = readNpmVersion(fullPath);
    const newVer = bumpSemver(oldVer, maxBump);
    writeNpmVersion(fullPath, newVer);
  }

  const section = buildChangelogEntries(entries, newCargoVersion);
  updateChangelog(section);

  deleteChanges(files);
  console.log(`\nDeleted ${files.length} changelog file(s)`);
  console.log(`\nDone. Workspace: ${cargoVersion} → ${newCargoVersion}`);
}

async function cmdPublish() {
  const npmPkg = join(ROOT, "packages/ikihsjs/package.json");
  const npmVersion = readNpmVersion(npmPkg);
  const { execSync } = await import("node:child_process");

  // Debug: check package info
  let published = "";
  try {
    published = execSync(`npm view ikihsjs version 2>&1 || true`, { encoding: "utf-8" }).trim();
  } catch {
    // not found
  }
  console.log(`npm view ikihsjs: ${published || "(not found)"}`);

  if (published === npmVersion) {
    console.log(`ikihsjs@${npmVersion} already published on npm — nothing to do`);
    return;
  }

  console.log(`Publishing ${npmPkg} version=${npmVersion}...`);

  // Print key env info for debugging OIDC
  console.log(`NODE_AUTH_TOKEN set: ${!!process.env.NODE_AUTH_TOKEN}`);
  console.log(`npmrc: ${execSync("npm config list", { encoding: "utf-8" }).trim()}`);

  execSync("pnpm build", { cwd: join(ROOT, "packages/ikihsjs"), stdio: "inherit" });

  const tag = `v${npmVersion}`;
  execSync("npm publish --access public --provenance --loglevel verbose", {
    cwd: join(ROOT, "packages/ikihsjs"),
    stdio: "inherit",
  });

  execSync(`git tag "${tag}"`, { stdio: "inherit" });
  execSync(`git push origin "${tag}"`, { stdio: "inherit" });
  console.log(`\nPublished ikihsjs@${npmVersion} and pushed tag ${tag}`);
}

// ─── Main ───

const cmd = process.argv[2] || "add";

switch (cmd) {
  case "add":
    await cmdAdd();
    break;
  case "version":
    cmdVersion();
    break;
  case "publish":
    await cmdPublish();
    break;
  default:
    console.error(`Unknown command: ${cmd}`);
    console.log("Usage: tsx scripts/changelog.ts <add|version|publish>");
    process.exit(1);
}
