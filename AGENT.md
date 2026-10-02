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

- `flomotion status` shows whether the desktop app is running, whether the page is connected, and which window it is addressing.
- `flomotion --window <name> ...`, or the `FLOMOTION_WINDOW` environment variable, drives a separate window with its own session. Use it when another agent may be using this machine; without it every agent shares one window.
- `flomotion agent` prints the current state as JSON: `role`, `system_prompt`, `tools`, `live_context`, and the open project and item. Read `system_prompt` and follow it. Pass `--reset` to start over from no project.
- `flomotion act <tool> '<json input>'` runs one tool and prints its result as JSON. Use `-f file.json` for large inputs. The output includes `changed`; when it is true, the response also carries a new `tools` list and possibly a new `system_prompt`, so re-read them.
- Some tools start a background job, for example validation, G-code generation or a long IK solve. `act` returns the job id straight away; run `flomotion job <id>` to wait for the result (it waits up to 90 seconds and can be repeated). To wait inside the `act` call instead, pass `--wait <seconds>`.
- `flomotion import <file>` brings a local file into the project. A `.step` or `.stp` model is added to the open workspace as new components, so open the workspace first. A `.zip` holding a MuJoCo MJCF model becomes a new workspace in the open project. Large files are handed over by path rather than copied, so a big archive is no slower than a small one.
- `flomotion act <tool> '<json input>' --file <path>` runs a tool that needs a local file, for example `flomotion act attach_robot_media '{"listing_id":"…","asset":"cover"}' --file ./banner.jpg` for a listing's banner. The file's type is filled into the input from its extension (put `content_type` in the input to override it), the bytes go straight to storage rather than through the tool protocol, and the command performs the whole exchange and prints the final result. Any tool whose schema takes `content_type` works this way. A listing's **video is not uploaded** like this - pass a YouTube or Vimeo link to `publish_robot` instead.
- `flomotion export <kind> [--id <component>] [--out <dir>]` writes an export of the focused item to disk and prints the paths. Kinds: `step` and `stl` for one component, `assembly_step` for the whole workspace, `gcode` for a CAM result, `kicad` for a schematic's symbols.
- `flomotion diagnose` prints what the window is showing (route, open project and item, whether it is signed in), any commands still running and for how long, and its recent command log. Run it when a command times out, reports that the window did not open something, or seems stuck, and include its output when you report the problem. If `diagnose` itself gets no answer, the window is frozen. Don't retry a create after a timeout: check `list_items` first, the item usually exists.
- `flomotion skill` prints this text.

The app opens automatically when needed. If the output says the user must sign in, sign the window in with `flomotion login`:

1. Run `flomotion login`. It prints a link. Give the link to the user and ask them to open it in a browser where they are signed in to FloMotion, approve, and send you the code it shows.
2. Run `flomotion login --code <code>`. The window is now signed in as the user; the output says who.

The code works once, for 10 minutes, and only for the window that printed the link. If it fails, start again from step 1. You never see or store the user's password or token.

## How the tools work

Tools depend on where you are. With no project open you can list, open, or create projects. Inside a project you manage files and open one. Opening a workspace, schematic, or scene switches the role and gives you that domain's tools. Navigation happens through tools such as `open_project`, `open_item`, and `close_item`.

Some tools run inside the app window, for example screenshots and simulations. Their results may contain a `files` list of local paths, usually images. Open those files to see them.

Each successful server action costs a small amount of the user's credits. Client-side tools and failed calls are free.
