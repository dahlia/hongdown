import { defineConfig } from "rolldown";

export default defineConfig({
  input: {
    extension: "src/extension.ts",
    "test/runTest": "tests/runTest.ts",
    "test/suite/index": "tests/suite/index.ts",
    "test/suite/formatter.test": "tests/suite/formatter.test.ts",
  },
  output: {
    dir: "dist",
    format: "cjs",
    sourcemap: true,
    entryFileNames: "[name].js",
  },
  external: ["vscode", "mocha", /^node:/],
  platform: "node",
});
