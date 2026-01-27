import { getLogger } from "@logtape/logtape";
import * as vscode from "vscode";

const logger = getLogger(["hongdown", "config"]);

/**
 * Check if the formatter should be enabled based on configuration.
 *
 * Disabled when hongdown.disable is set to true.
 * Otherwise, enabled by default.
 */
export function isFormatterEnabled(): boolean {
  const config = vscode.workspace.getConfiguration("hongdown");
  const disableSetting = config.inspect<boolean>("disable");

  logger.debug`isFormatterEnabled() called`;
  logger.debug`  disableSetting: ${disableSetting}`;

  // Check if disabled at any level
  const isDisabled =
    disableSetting?.globalValue === true ||
    disableSetting?.workspaceValue === true ||
    disableSetting?.workspaceFolderValue === true;

  logger.debug`  isDisabled: ${isDisabled}`;

  if (isDisabled) {
    logger.debug`  returning: false (disabled)`;
    return false;
  }

  logger.debug`  returning: true`;
  return true;
}
