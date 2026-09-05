# FloMotion Desktop

Desktop shell and CLI for [FloMotion](https://flomotion.app), the AI-driven robot design platform.

It lets you drive FloMotion with your own AI agent (Claude Code, Codex, Gemini CLI, Cursor, or anything that can run a command) on your own agent subscription. The app hosts the FloMotion web client so the agent can run client-side tools such as screenshots and simulations, and so you can watch the agent work.

Status: working end to end on Windows and Linux against the hosted client. macOS passes its tests and a smoke run in CI, where the window opens, the page connects and the WebGL screenshot tool renders; it has not been used on a physical mac yet, reports welcome. The hosted client is closed source and implements the page side of the protocol; the shell forwards `agent`, `act`, `job`, `import` and `export` commands to it over a Tauri event and reads back the answers.

## Installing

Archives for Linux, macOS and Windows are on the [releases page](https://github.com/flomotion-app/flomotion-desktop/releases). The install scripts download the latest one and put `flomotion` on the path:

```
curl -fsSL https://raw.githubusercontent.com/flomotion-app/flomotion-desktop/main/install.sh | sh
```

```
irm https://raw.githubusercontent.com/flomotion-app/flomotion-desktop/main/install.ps1 | iex
```

Manual install is the same thing by hand. Each archive contains one folder `flomotion` with the binary, `AGENT.md`, `README.md` and `LICENSE`:

| Platform | Archive | Steps |
|---|---|---|
| Linux x86_64 | `flomotion-linux-x86_64.tar.gz` | `tar -xzf`, copy `flomotion/flomotion` somewhere on PATH, `sudo apt-get install libwebkit2gtk-4.1-0` |
| macOS arm64 | `flomotion-macos-arm64.tar.gz` | `tar -xzf`, copy `flomotion/flomotion` somewhere on PATH |
| macOS x86_64 | `flomotion-macos-x86_64.tar.gz` | same |
| Windows x86_64 | `flomotion-windows-x86_64.zip` | extract, put `flomotionlomotion.exe` on PATH |

The archives are not code signed. The macOS binaries are ad-hoc signed, which is enough to run a file fetched with curl or tar rather than a browser download. Windows needs the WebView2 runtime, present on Windows 11 and most Windows 10 machines. Anyone who prefers not to run an unsigned binary can build from source, see below.

## For agents

Point the agent at [AGENT.md](AGENT.md): it covers installing the binary and every command. The home page copies a one-line prompt that does exactly that. After install, `flomotion skill` prints the same text.

## Layout

- `src-tauri/` Rust. One binary: with no arguments it opens the FloMotion window, with a subcommand it acts as the CLI and talks to the running window over a local socket.
- `src/` the local test page used in development. The real UI is the hosted web client.

## Commands

```
flomotion                 open the window
flomotion status          is the app running, is the page connected
flomotion agent [--reset] print role, system prompt, tools, live context
flomotion act <tool> '{"json": "input"}'
flomotion act <tool> -f input.json
flomotion act <tool> --wait 300     wait longer for a batch job
flomotion job <id> [--wait N]       keep waiting for a batch job
flomotion import <file.step>        import a STEP model into the open workspace
flomotion export <kind> [--id X] [--out DIR]   write STEP, STL, G-code or KiCad symbols to disk
flomotion skill           print the agent instructions
```

## Development

Requires Rust stable and Node 20+.

```
npm install
FLOMOTION_URL=local npm run tauri dev
```

The window loads flomotion.app by default and opens on `/projects`. To point it elsewhere, create `~/.flomotion/config.json`:

```json
{ "web_url": "https://feature.flomotion.app" }
```

The `FLOMOTION_URL` environment variable overrides the file. Use `local` to load the test page in `src/`.

Debugging the page on Windows: set `FLOMOTION_WEBVIEW_ARGS=--remote-debugging-port=9222` before launching, then attach any Chrome DevTools Protocol client to port 9222.

Tests:

```
cd src-tauri && cargo test
```

## Building

```
npm install
npm run tauri build
```

## License

MIT, see [LICENSE](LICENSE).
