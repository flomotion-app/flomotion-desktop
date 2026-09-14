import { convertFileSrc, invoke } from "@tauri-apps/api/core";
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

interface ImportPayload {
  name: string;
  path?: string;
  data_base64?: string;
}

class ImportedFile {
  async resolve(payload: ImportPayload): Promise<{ file: File; via: string; attempts: string[] }> {
    const attempts: string[] = [];
    if (payload.data_base64) {
      const binary = atob(payload.data_base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return { file: new File([bytes], payload.name), via: "inline", attempts: ["inline"] };
    }
    if (!payload.path) throw new Error("import payload has neither path nor data_base64");
    try {
      return { file: await this.fetchFile(convertFileSrc(payload.path), payload.name), via: "assetProtocol", attempts };
    } catch (e) {
      attempts.push(`assetProtocol failed: ${e instanceof Error ? e.message : String(e)}`);
    }
    const url = await invoke<string>("cli_file_url", { path: payload.path });
    return { file: await this.fetchFile(url, payload.name), via: "command", attempts };
  }

  private async fetchFile(url: string, name: string): Promise<File> {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`http ${response.status}`);
    const blob = await response.blob();
    return new File([blob], name, { type: blob.type });
  }
}

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
  private readonly imports = new ImportedFile();

  async start(): Promise<void> {
    const bridge = new DesktopBridge((command, payload) => this.echo(command, payload));
    await bridge.connect();
    this.status.textContent = "Connected, waiting for your agent";
  }

  private async echo(command: string, payload: unknown): Promise<unknown> {
    this.log.textContent += `${command} ${JSON.stringify(payload).slice(0, 200)}\n`;
    if (command === "fail") throw new Error("requested failure");
    if (command === "import") return this.importFile(payload as ImportPayload);
    return { echo: command, payload };
  }

  private async importFile(payload: ImportPayload): Promise<unknown> {
    const started = performance.now();
    const { file, via, attempts } = await this.imports.resolve(payload);
    const result = { via, attempts, name: file.name, size: file.size, type: file.type, ms: Math.round(performance.now() - started) };
    this.log.textContent += `  -> ${JSON.stringify(result)}\n`;
    return result;
  }
}

window.addEventListener("DOMContentLoaded", () => void new LocalTestPage().start());
