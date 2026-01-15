import { spawnSync } from "node:child_process";
import * as os from "node:os";
import * as path from "node:path";

import { getLogger } from "@logtape/logtape";
import * as vscode from "vscode";

const logger = getLogger(["hongdown", "formatter"]);

/**
 * Format Markdown content using the hongdown CLI.
 *
 * @param content - The Markdown content to format.
 * @param cwd - The working directory to run hongdown in. If empty, uses home directory.
 * @param executablePath - Path to hongdown executable. Defaults to "hongdown".
 * @returns The formatted Markdown content.
 */
export function formatWithCli(
  content: string,
  cwd: string,
  executablePath: string = "hongdown",
): string {
  // Use home directory as fallback if cwd is invalid
  const workingDir = cwd && path.isAbsolute(cwd) ? cwd : os.homedir();

  const shell = process.env.SHELL || "/bin/bash";
  const result = spawnSync(shell, ["-i", "-c", `${executablePath} --stdin`], {
    cwd: workingDir,
    input: content,
    env: {
      ...process.env,
    },
    encoding: "utf-8",
  });

  if (result.error) {
    throw new Error(
      `Failed to start hongdown: ${result.error.message}. ` +
        "Make sure hongdown is installed and available in PATH, " +
        "or set hongdown.path in VS Code settings.",
    );
  }

  if (result.status !== 0) {
    const errorMessage =
      result.stderr?.trim() || `hongdown exited with code ${result.status}`;
    throw new Error(errorMessage);
  }

  return result.stdout;
}

/**
 * Get the workspace folder for a document URI.
 * Falls back to the first workspace folder, or empty string for untitled documents.
 */
function getWorkspaceFolder(uri: vscode.Uri): string {
  // For file URIs, try to get the workspace folder
  if (uri.scheme === "file") {
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
    if (workspaceFolder) {
      return workspaceFolder.uri.fsPath;
    }
    // Fall back to the document's directory
    const filePath = uri.fsPath;
    const lastSlash = Math.max(
      filePath.lastIndexOf("/"),
      filePath.lastIndexOf("\\"),
    );
    if (lastSlash > 0) {
      return filePath.substring(0, lastSlash);
    }
  }

  // For untitled documents or when no workspace folder found,
  // use the first workspace folder if available
  if (
    vscode.workspace.workspaceFolders &&
    vscode.workspace.workspaceFolders.length > 0
  ) {
    return vscode.workspace.workspaceFolders[0].uri.fsPath;
  }

  // Return empty string - formatter will use home directory as fallback
  return "";
}

/**
 * Hongdown Markdown formatter provider.
 */
export class HongdownFormattingProvider
  implements vscode.DocumentFormattingEditProvider
{
  provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    _options: vscode.FormattingOptions,
    token: vscode.CancellationToken,
  ): vscode.TextEdit[] {
    logger.debug`provideDocumentFormattingEdits() called for: ${document.uri.fsPath}`;

    if (token.isCancellationRequested) {
      logger.debug`Cancellation requested at start`;
      return [];
    }

    const text = document.getText();
    if (!text) {
      return [];
    }

    const cwd = getWorkspaceFolder(document.uri);

    // Read hongdown.path setting
    const config = vscode.workspace.getConfiguration("hongdown");
    const execPath = config.get<string>("path") || "hongdown";

    logger.info`Formatting: ${document.uri.fsPath}`;
    logger.debug`  workingDir: ${cwd || "(empty, will use home directory)"}`;
    logger.debug`  execPath: ${execPath}`;

    try {
      const formatted = formatWithCli(text, cwd, execPath);

      if (token.isCancellationRequested) {
        logger.debug`Formatting cancelled: ${document.uri.fsPath}`;
        return [];
      }

      // If no changes, return empty array
      if (formatted === text) {
        logger.debug`No changes needed: ${document.uri.fsPath}`;
        return [];
      }

      logger.info`Formatted successfully: ${document.uri.fsPath}`;

      // Create a full document replacement edit
      const fullRange = new vscode.Range(
        document.positionAt(0),
        document.positionAt(text.length),
      );

      return [vscode.TextEdit.replace(fullRange, formatted)];
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Unknown error occurred";
      logger.error`Error formatting ${document.uri.fsPath}: ${message}`;
      vscode.window.showErrorMessage(`Hongdown: ${message}`);
      return [];
    }
  }
}
