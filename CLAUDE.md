Flomotion desktop: Tauri 2 shell plus CLI for driving Flomotion with an external agent.

Rules:
- Rust for the shell and CLI, TypeScript for the small local page. No frameworks in the local page.
- One binary: no subcommand opens the window, subcommands act as the CLI and talk to the running window over a local socket.
- The window loads the hosted Flomotion web client; this repo never contains the web client source.
- The CLI never holds credentials. Auth lives in the webview session.
- Client tools run in the page, results are returned to the CLI, nothing is recorded server-side.
- Dependency injection, no globals, no static helpers. Modules short and focused.
- No comments in code, no em dashes in text.
- Commit messages short, no authorship lines.
- Never commit or push until explicitly told to. Approval for one changeset never carries over to the next.
- Don't use the AskUserQuestion tool. When a decision is needed, list the options inline and let me pick.
