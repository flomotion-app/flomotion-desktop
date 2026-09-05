import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface CliRequest {
  id: number;
  command: string;
  payload: unknown;
}

interface CliResponse {
  ok: boolean;
  result: unknown;
  error?: string;
}

type CommandHandler = (command: string, payload: unknown) => Promise<unknown>;

class DesktopBridge {
  constructor(private readonly handler: CommandHandler) {}

  async connect(): Promise<void> {
    await listen<CliRequest>("cli-request", (event) => void this.answer(event.payload));
    await invoke("cli_ready");
  }

  private async answer(request: CliRequest): Promise<void> {
    const response = await this.handle(request);
    await invoke("cli_respond", { id: request.id, response });
  }

  private async handle(request: CliRequest): Promise<CliResponse> {
    try {
      return { ok: true, result: await this.handler(request.command, request.payload) };
    } catch (e) {
      return { ok: false, result: null, error: e instanceof Error ? e.message : String(e) };
    }
  }
}

class LocalTestPage {
  private readonly status = document.getElementById("status")!;
  private readonly log = document.getElementById("log")!;

  async start(): Promise<void> {
    const bridge = new DesktopBridge((command, payload) => this.echo(command, payload));
    await bridge.connect();
    this.status.textContent = "Connected, waiting for your agent";
  }

  private async echo(command: string, payload: unknown): Promise<unknown> {
    this.log.textContent += `${command} ${JSON.stringify(payload)}\n`;
    if (command === "fail") throw new Error("requested failure");
    return { echo: command, payload };
  }
}

window.addEventListener("DOMContentLoaded", () => void new LocalTestPage().start());
