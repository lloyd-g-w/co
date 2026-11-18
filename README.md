# co

A coding agent harness: a Rust server that runs agents and stores users,
settings and sessions in Postgres; tool hosts that run tools on your machines
(switchable mid-session); and frontends: a terminal UI, a SolidJS web app and
a Tauri desktop app. See [ARCHITECTURE.md](ARCHITECTURE.md).

## Development

Everything you need is in the Nix dev shell (`nix develop`, or `direnv allow`
with the included `.envrc`): the Rust toolchain from `rust-toolchain.toml`,
Node 26, pnpm, just, sqlx-cli, cargo-tauri, psql and the Tauri system libraries.

```sh
just install    # pnpm install
just db         # dev Postgres in Docker (postgres://co:co@localhost:5432/co)
just dev        # server (:7420) + web (:5173)
just tool-host  # make this machine a tool host
just tui        # terminal UI
just desktop    # desktop app
just check      # fmt, clippy, lint, typecheck
just test
just codegen    # regenerate TypeScript protocol types after changing crates/protocol
```

Run `just` for the full list.
