import * as vscode from "vscode";
import { inspect } from "node:util";

export const log = new (class {
  private enabled = true;
  private readonly output = vscode.window.createOutputChannel("REVIS");

  setEnabled(yes: boolean): void {
    log.enabled = yes;
  }

  // Hint: the type [T, ...T[]] means a non-empty array
  debug(...msg: [unknown, ...unknown[]]): void {
    if (!log.enabled) {
      return;
    }
    log.write("DEBUG", ...msg);
  }

  info(...msg: [unknown, ...unknown[]]): void {
    log.write("INFO", ...msg);
  }

  warn(...msg: [unknown, ...unknown[]]): void {
    debugger;
    log.write("WARN", ...msg);
  }

  error(...msg: [unknown, ...unknown[]]): void {
    debugger;
    log.write("ERROR", ...msg);
    log.output.show(true);
  }

  private write(label: string, ...messageParts: unknown[]): void {
    const message = messageParts.map(log.stringify).join(" ");
    const dateTime = new Date().toLocaleString();
    log.output.appendLine(`${label} [${dateTime}]: ${message}`);
  }

  private stringify(val: unknown): string {
    if (typeof val === "string") {
      return val;
    }
    return inspect(val, {
      colors: false,
      depth: 6, // heuristic
    });
  }
})();
