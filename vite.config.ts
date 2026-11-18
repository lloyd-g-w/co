// Workspace-wide Vite+ settings: lint (Oxlint) and format (Oxfmt) for every
// TS package. Package-specific Vite config lives in each app's vite.config.ts.
import { defineConfig } from "vite-plus";

const ignorePatterns = [
  "**/dist/**",
  "**/node_modules/**",
  "target/**",
  "apps/desktop/src-tauri/**",
  "packages/protocol/src/generated/**",
];

export default defineConfig({
  fmt: {
    // Rust-side files (Cargo.toml, rustfmt) and prose are not Oxfmt's business.
    ignorePatterns: [
      ...ignorePatterns,
      "**/*.toml",
      "**/*.md",
      "crates/**",
      "apps/cli/**",
      "apps/tui/**",
    ],
    printWidth: 100,
    sortPackageJson: true,
  },
  lint: {
    ignorePatterns,
    plugins: ["typescript", "unicorn", "import"],
    options: {
      typeAware: true,
      typeCheck: true,
    },
    rules: {
      "typescript/no-floating-promises": "error",
      "typescript/consistent-type-imports": "error",
    },
    overrides: [
      {
        files: ["**/*.test.ts", "**/*.test.tsx"],
        plugins: ["vitest"],
      },
    ],
  },
});
