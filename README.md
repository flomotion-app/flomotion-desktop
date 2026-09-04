# Flomotion Desktop

Desktop shell and CLI for [Flomotion](https://flomotion.app), the AI-driven robot design platform.

It lets you drive Flomotion with your own AI agent (Claude Code, Codex, Gemini CLI, Cursor, or anything that can run a command) using your own agent subscription. The app hosts the Flomotion web client so the agent can run client-side tools such as screenshots and simulations, and so you can watch the agent work.

Status: early development. Nothing here is usable yet.

## Planned layout

- `src-tauri/` Rust shell: opens the Flomotion web client in a native window, exposes a local socket, and doubles as the `flomotion` CLI.
- `src/` minimal local page shown before the web client loads.

## Building from source

Requires Rust (stable) and Node 20+.

```
npm install
npm run tauri build
```

## License

MIT, see [LICENSE](LICENSE).
