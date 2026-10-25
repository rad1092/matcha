#!/usr/bin/env node
// Print connection settings for this installation; never edit a host's config.
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const server = [join(here, "server", "server.mjs"), join(here, "server.mjs")].find(existsSync);
if (!server) throw new Error("Cannot find the matcha MCP server next to this configuration helper.");
console.log(JSON.stringify({
  mcpServers: { matcha: { command: process.execPath, args: [server] } },
}, null, 2));
