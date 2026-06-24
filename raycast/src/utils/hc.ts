import { execSync, spawnSync } from "child_process";
import { existsSync } from "fs";

const HC_CANDIDATES = [
  `${process.env.HOME}/.cargo/bin/hc`,
  "/usr/local/bin/hc",
  "/opt/homebrew/bin/hc",
];

function findHcPath(): string {
  for (const p of HC_CANDIDATES) {
    if (p && existsSync(p)) return p;
  }
  throw new Error("hc command not found. Please install holecard.");
}

const HC = findHcPath();

export interface Hand {
  name: string;
  cards: string[];
}

export interface TotpResult {
  code: string;
  remainingSeconds: number;
}

function run(args: string): string {
  return execSync(`${HC} ${args}`, { timeout: 10000 }).toString();
}

function spawn(args: string[]): string {
  const result = spawnSync(HC, args, { timeout: 10000, encoding: "utf-8" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error((result.stderr as string) || `hc ${args[0]} failed`);
  return result.stdout as string;
}

function parseHandList(output: string): Hand[] {
  const hands: Hand[] = [];
  let current: Hand | null = null;

  for (const line of output.split("\n")) {
    const handMatch = line.match(/^\s+•\s+(.+)$/);
    if (handMatch) {
      if (current) hands.push(current);
      current = { name: handMatch[1].trim(), cards: [] };
      continue;
    }

    const cardsMatch = line.match(/^\s+Cards:\s+(.+)$/);
    if (cardsMatch && current) {
      current.cards = cardsMatch[1].split(",").map((c) => c.trim());
    }
  }

  if (current) hands.push(current);
  return hands;
}

export function listHands(): Hand[] {
  return parseHandList(run("hand list")).filter((h) => h.name !== "totp");
}

export function listTotpServices(): string[] {
  const totpHand = parseHandList(run("hand list")).find((h) => h.name === "totp");
  return totpHand ? totpHand.cards : [];
}

export function copyCard(handName: string, cardName: string): void {
  spawn(["hand", "get", handName, "--clip", cardName]);
}

export interface TotpEntry {
  service: string;
  result: TotpResult | null;
  error: string | null;
}

export function getTotpCode(serviceName: string): TotpResult {
  const output = run(`totp get "${serviceName}"`);
  const match = output.match(/TOTP Code:\s+(\d+)\s+\(valid for (\d+) seconds\)/);
  if (!match) throw new Error(`Failed to parse TOTP output: ${output}`);
  return {
    code: match[1],
    remainingSeconds: parseInt(match[2], 10),
  };
}

export function addHand(name: string, cards: Record<string, string>, note = ""): void {
  const flags = Object.entries(cards)
    .filter(([k, v]) => k.trim() && v.trim())
    .flatMap(([k, v]) => ["-f", `${k}=${v}`]);
  spawn(["hand", "add", name, ...flags, "--note", note]);
}

export function upsertCard(handName: string, key: string, value: string): void {
  spawn(["hand", "edit", handName, "-f", `${key}=${value}`]);
}

export function removeCard(handName: string, key: string): void {
  spawn(["hand", "edit", handName, "--rm-card", key]);
}

export function renameCardKey(handName: string, oldKey: string, newKey: string): void {
  const value = spawn(["read", `hc://${handName}/${oldKey}`]).trim();
  spawn(["hand", "edit", handName, "-f", `${newKey}=${value}`, "--rm-card", oldKey]);
}

export function removeHand(handName: string): void {
  run(`hand remove "${handName}"`);
}

export function fetchTotpEntry(service: string): TotpEntry {
  try {
    return { service, result: getTotpCode(service), error: null };
  } catch (e) {
    return { service, result: null, error: e instanceof Error ? e.message : String(e) };
  }
}
