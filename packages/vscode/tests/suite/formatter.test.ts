import * as assert from "node:assert";
import * as path from "node:path";
import { spawnSync } from "node:child_process";
import * as vscode from "vscode";

/**
 * Get the path to hongdown binary.
 * Uses HONGDOWN_PATH env var if set, otherwise falls back to "hongdown".
 */
function getHongdownPath(): string {
  return process.env.HONGDOWN_PATH || "hongdown";
}

/**
 * Format content using hongdown CLI directly.
 * Used as the reference implementation for test comparisons.
 */
function formatWithCliDirectly(content: string, cwd: string): string {
  const hongdownPath = getHongdownPath();
  // Use home directory as fallback if cwd is invalid
  const os = require("os");
  const workingDir = cwd && path.isAbsolute(cwd) ? cwd : os.homedir();

  const result = spawnSync(hongdownPath, ["--stdin"], {
    cwd: workingDir,
    input: content,
    encoding: "utf-8",
  });

  if (result.error) {
    throw new Error(`Failed to run hongdown: ${result.error.message}`);
  }

  if (result.status !== 0) {
    throw new Error(
      result.stderr || `hongdown exited with code ${result.status}`,
    );
  }

  return result.stdout;
}

/**
 * Apply text edits to a document and return the result.
 */
async function applyEditsToDocument(
  doc: vscode.TextDocument,
  edits: vscode.TextEdit[],
): Promise<string> {
  if (!edits || edits.length === 0) {
    return doc.getText();
  }

  const workspaceEdit = new vscode.WorkspaceEdit();
  workspaceEdit.set(doc.uri, edits);
  await vscode.workspace.applyEdit(workspaceEdit);

  return doc.getText();
}

suite("Hongdown Formatter", () => {
  const testTimeout = 30000;

  suiteSetup(async function () {
    this.timeout(testTimeout);

    const config = vscode.workspace.getConfiguration("hongdown");

    // Ensure formatter is enabled for tests
    await config.update("disable", false, vscode.ConfigurationTarget.Global);

    // Configure hongdown.path if HONGDOWN_PATH env var is set
    const hongdownPath = getHongdownPath();
    if (hongdownPath !== "hongdown") {
      await config.update(
        "path",
        hongdownPath,
        vscode.ConfigurationTarget.Global,
      );
    }

    // Wait for extension to activate
    const ext = vscode.extensions.getExtension("hongdown.hongdown-vscode");
    if (ext && !ext.isActive) {
      await ext.activate();
    }
  });

  test("formats markdown same as CLI", async function () {
    this.timeout(testTimeout);

    const content = "# Hello\n\n* Item 1\n* Item 2";

    // Create a new untitled document
    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content,
    });

    await vscode.window.showTextDocument(doc);

    // Execute format command via VS Code
    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      doc.uri,
      { tabSize: 2, insertSpaces: true },
    );

    // Apply edits to get the formatted result
    const formatted = await applyEditsToDocument(doc, edits ?? []);

    // Get expected result from CLI directly
    // Use home directory to match the extension's behavior for untitled documents
    const os = require("os");
    const cwd = os.homedir();
    const expected = formatWithCliDirectly(content, cwd);

    assert.strictEqual(formatted, expected);

    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  });

  test("handles empty document", async function () {
    this.timeout(testTimeout);

    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content: "",
    });

    await vscode.window.showTextDocument(doc);

    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      doc.uri,
      { tabSize: 2, insertSpaces: true },
    );

    // Should handle gracefully with no edits
    assert.ok(
      !edits || edits.length === 0,
      "Should have no edits for empty document",
    );

    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  });

  test("formats complex markdown same as CLI", async function () {
    this.timeout(testTimeout);

    const content = `# Main Title

## Section One

Here is some text with a [link](https://example.com).

### Subsection

* First item
* Second item
  * Nested item

1. Ordered item
2. Another item

\`\`\`javascript
const x = 1;
\`\`\`
`;

    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content,
    });

    await vscode.window.showTextDocument(doc);

    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      doc.uri,
      { tabSize: 2, insertSpaces: true },
    );

    // Apply edits to get the formatted result
    const formatted = await applyEditsToDocument(doc, edits ?? []);

    // Use home directory to match the extension's behavior for untitled documents
    const os = require("os");
    const cwd = os.homedir();
    const expected = formatWithCliDirectly(content, cwd);

    assert.strictEqual(formatted, expected);

    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  });

  test("disabled when hongdown.disable is true", async function () {
    this.timeout(testTimeout);

    const config = vscode.workspace.getConfiguration("hongdown");

    // Disable the formatter
    await config.update("disable", true, vscode.ConfigurationTarget.Global);

    const content = "# Hello\n\n* Item 1\n* Item 2";

    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content,
    });

    await vscode.window.showTextDocument(doc);

    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      doc.uri,
      { tabSize: 2, insertSpaces: true },
    );

    // Should have no edits when disabled
    assert.ok(
      !edits || edits.length === 0,
      "Should have no edits when disabled",
    );

    // Re-enable for other tests
    await config.update("disable", false, vscode.ConfigurationTarget.Global);

    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  });
});
