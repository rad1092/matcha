#!/usr/bin/env node
// Writes analysis/data/opcodes.json: the mnemonic of every SM83 opcode
// (operands shown as zero), from matcha's own disassembler. report.py uses
// it to label the instruction-mix tables.
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadMatcha } from "../web/matcha.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const mod = await loadMatcha(readFileSync(join(root, "web/matcha.wasm")));
const gb = mod.create(readFileSync(join(root, "web/roms/libbet.gb")));
const names = (cb) => Array.from({ length: 256 }, (_, op) => gb.opcodeText(op, cb));
writeFileSync(join(root, "analysis/data/opcodes.json"), JSON.stringify({ base: names(false), cb: names(true) }, null, 1) + "\n");
