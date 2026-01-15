import type { LogRecord, Sink } from "@logtape/logtape";
import type { LogOutputChannel } from "vscode";

/**
 * Create a LogTape sink that writes to a VS Code LogOutputChannel.
 * LogOutputChannel provides colored output based on log level.
 */
export function getLogOutputChannelSink(channel: LogOutputChannel): Sink {
  return (record: LogRecord) => {
    const category = record.category.join("\xb7");
    const messageBody = record.message
      .map((part) => (typeof part === "string" ? part : JSON.stringify(part)))
      .join("");
    const message = `[${category}] ${messageBody}`;

    switch (record.level) {
      case "debug":
        channel.debug(message);
        break;
      case "info":
        channel.info(message);
        break;
      case "warning":
        channel.warn(message);
        break;
      case "error":
      case "fatal":
        channel.error(message);
        break;
      default:
        channel.trace(message);
    }
  };
}
