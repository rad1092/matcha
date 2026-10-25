# matcha — Game Boy for AI agents

AI agents can play, inspect and debug Game Boy and native Game Boy Color
software through 17 MCP tools. The package includes the emulator, four
open-source sample games, an operator guide and three optional workflow skills.

## Connect

1. Extract `matcha-ai.zip` to a permanent folder.
2. Install Node.js 18 or newer if needed. No `npm install`, API key or Rust
   build is needed for this package.
3. In the extracted `matcha-ai` folder, run:

   ```sh
   node configure.mjs
   ```

4. Add the printed `matcha` entry to your AI application's MCP settings.
   For applications with a configuration form, select **stdio** and use the
   printed `command` and `args` values. `mcpServers` is a common JSON layout;
   some hosts use another key or UI. Merge the entry with existing settings.
5. Connect the server, then ask the AI to call `load_rom` with
   `{"sample":"libbet"}` and describe the returned screen.

The helper prints absolute Node and server paths for the current machine.
Run it again after moving the extracted folder or changing Node installations.
The server can be launched from any working directory. Use absolute paths for
your own ROMs; relative ROM paths resolve against the server's working directory.

The AI application must support local MCP servers, tool calls and one of these
protocol versions: `2024-11-05`, `2025-03-26`, `2025-06-18`, `2025-11-25`.
For visual play, it must also pass MCP image results to an image-capable model.
Text tools can inspect registers and memory without image understanding.
Applications that accept only remote HTTP MCP endpoints need an adapter; this
package provides a local stdio server. Compatibility with a particular AI
application must be checked in that application.

## Operate

Read [AI_GUIDE.md](AI_GUIDE.md) for the complete Korean operator manual,
copyable tool examples, all 17 tools, debugging workflows and test cleanup.
The files under `skills/` contain English play, debugging and RAM-search
workflows. A host can load them as skills if supported, or an agent can read
them as ordinary instructions; automatic skill discovery is host-specific.

Example request to an AI:

> matcha의 AI_GUIDE.md를 읽고 libbet 샘플을 실행해줘. 시작 화면을 확인하고
> Start를 눌러 게임 진행을 확인해줘. 관찰한 결과와 입력 순서를 기록해줘.

## 한국어 빠른 시작

압축을 푼 뒤 `node configure.mjs`를 실행하고, 출력된 서버 설정을 사용하는
AI 앱에 추가하세요. 앱의 입력 폼을 쓴다면 전송 방식은 `stdio`, 명령과 인자는
출력된 `command`, `args` 값을 사용합니다. 그다음 AI에게 조작서를 읽고
`load_rom({"sample":"libbet"})`을 호출하도록 요청하면 됩니다.

MCP 서버와 에뮬레이터는 로컬에서 실행되며 직접 네트워크 요청을 하지 않습니다.
AI 앱이 도구 결과를 모델에 전달하는 방식은 해당 앱의 설정을 따릅니다.
게임보이·게임보이 컬러 ROM을 지원하며, 일부 특수 카트리지와 정밀 타이밍은
아직 지원하지 않습니다. 기본 게임 네 개는 샘플이며 다른 `.gb` / `.gbc` 파일도
불러올 수 있습니다.

## Package contents and verification

- `server/`: MCP server, JavaScript bindings, WebAssembly core and sample ROMs.
- `configure.mjs`: prints connection settings without changing application settings.
- `AI_GUIDE.md`, `skills/`: operating instructions.
- `package.json`: package metadata and explicit JavaScript module format.
- `SHA256SUMS`: SHA-256 checksums for every other file in the generic ZIP.
- `LICENSE-MIT`, `LICENSE-APACHE`, `server/roms/LICENSES.md`: code and game licences.

On macOS, verify the extracted files with `shasum -a 256 -c SHA256SUMS`;
on Linux, use `sha256sum -c SHA256SUMS`.

To rebuild from the source checkout, run `node scripts/build-ai-package.mjs`.
The output is `dist/matcha-ai.zip`. Rebuild `web/matcha.wasm` first if the Rust
core or WASM exports changed. Build staging directories are removed automatically.
