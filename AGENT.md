# Driving FloMotion from an AI agent

FloMotion is a robot design platform: mechanical CAD, electronics schematics with simulation, and control scenes with physics. The `flomotion` command lets you, the agent, use its tools directly with the user's account. This file is both the install guide and the usage guide. Follow it top to bottom.

## Install

Check first: if `flomotion status` already prints JSON, skip to Commands.

Pick the line for the user's platform and run it. Each downloads the latest release archive from GitHub and puts `flomotion` on the path.

- Linux or macOS: `curl -fsSL https://raw.githubusercontent.com/flomotion-app/flomotion-desktop/main/install.sh | sh`
- Windows, in PowerShell: `irm https://raw.githubusercontent.com/flomotion-app/flomotion-desktop/main/install.ps1 | iex`

If the script cannot be used, do it by hand: download the matching archive from https://github.com/flomotion-app/flomotion-desktop/releases/latest (`flomotion-linux-x86_64.tar.gz`, `flomotion-macos-arm64.tar.gz`, `flomotion-macos-x86_64.tar.gz` or `flomotion-windows-x86_64.zip`), extract it, and copy `flomotion/flomotion` (or `flomotion.exe`) to a directory on the path. Linux also needs WebKitGTK: `sudo apt-get install libwebkit2gtk-4.1-0` on Debian and Ubuntu. Building from source needs Rust and Node and is described in the repository README.

The archives are not code signed. On macOS the binary is ad-hoc signed, which runs fine when fetched with curl or tar. If the user prefers, build from source instead.

Then run `flomotion status`. Open a new terminal first if the path was just changed. On the first real command the FloMotion window opens; ask the user to sign in there if the output says so.

## Commands

- `flomotion status` shows whether the desktop app is running and the page is connected.
- `flomotion agent` prints the current state as JSON: `role`, `system_prompt`, `tools`, `live_context`, and the open project and item. Read `system_prompt` and follow it. Pass `--reset` to start over from no project.
- `flomotion act <tool> '<json input>'` runs one tool and prints its result as JSON. Use `-f file.json` for large inputs. The output includes `changed`; when it is true, the response also carries a new `tools` list and possibly a new `system_prompt`, so re-read them.
- Some tools start a background job, for example validation or G-code generation. `act` waits up to 90 seconds for it (change with `--wait`). If the output says the job is still pending, run `flomotion job <id>` to keep waiting.
- `flomotion import <file.step>` imports a STEP model into the open workspace as new components. Open the workspace first.
- `flomotion export <kind> [--id <component>] [--out <dir>]` writes an export of the focused item to disk and prints the paths. Kinds: `step` and `stl` for one component, `assembly_step` for the whole workspace, `gcode` for a CAM result, `kicad` for a schematic's symbols.
- `flomotion skill` prints this text.

The app opens automatically when needed. If the output says the user must sign in, ask them to sign in inside the FloMotion window and try again.

## How the tools work

Tools depend on where you are. With no project open you can list, open, or create projects. Inside a project you manage files and open one. Opening a workspace, schematic, or scene switches the role and gives you that domain's tools. Navigation happens through tools such as `open_project`, `open_item`, and `close_item`.

Some tools run inside the app window, for example screenshots and simulations. Their results may contain a `files` list of local paths, usually images. Open those files to see them.

Each successful server action costs a small amount of the user's credits. Client-side tools and failed calls are free.
