import { configure, getLogger, reset } from "@logtape/logtape";
import * as vscode from "vscode";

import { isFormatterEnabled } from "./config";
import { HongdownFormattingProvider } from "./formatter";
import { getLogOutputChannelSink } from "./sink";

let formatterDisposables: vscode.Disposable[] = [];

const logger = getLogger(["hongdown"]);
const configLogger = getLogger(["hongdown", "config"]);
const formatterLogger = getLogger(["hongdown", "formatter"]);

/**
 * Register the formatter if enabled, or unregister if disabled.
 */
function registerFormatter(): void {
  // Dispose existing registrations
  for (const disposable of formatterDisposables) {
    disposable.dispose();
  }
  formatterDisposables = [];

  if (!isFormatterEnabled()) {
    formatterLogger.info`Formatter disabled, not registering`;
    return;
  }

  formatterLogger.info`Registering formatter`;

  const provider = new HongdownFormattingProvider();

  // Register for file scheme
  formatterDisposables.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      { language: "markdown", scheme: "file" },
      provider,
    ),
  );

  // Register for untitled scheme
  formatterDisposables.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      { language: "markdown", scheme: "untitled" },
      provider,
    ),
  );
}

/**
 * Extension activation.
 */
export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  const outputChannel = vscode.window.createOutputChannel("Hongdown", {
    log: true,
  });
  context.subscriptions.push(outputChannel);

  // Configure LogTape
  await configure({
    sinks: {
      outputChannel: getLogOutputChannelSink(outputChannel),
    },
    loggers: [
      {
        category: ["hongdown"],
        lowestLevel: "debug",
        sinks: ["outputChannel"],
      },
    ],
  });

  logger.info`Hongdown extension activated`;

  // Initial registration
  registerFormatter();

  // Listen for configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("hongdown.disable")) {
        configLogger.info`hongdown.disable changed, re-registering formatter`;
        registerFormatter();
      }
    }),
  );

  // Clean up formatter disposables on deactivation
  context.subscriptions.push({
    dispose: () => {
      for (const disposable of formatterDisposables) {
        disposable.dispose();
      }
      formatterDisposables = [];
    },
  });
}

/**
 * Extension deactivation.
 */
export async function deactivate(): Promise<void> {
  logger.info`Hongdown extension deactivated`;
  await reset();
}
