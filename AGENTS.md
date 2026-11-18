# Notes for coding agents

- Read ARCHITECTURE.md first. The server owns all logic; frontends render and
  take input only.
- Work in the Nix dev shell (`nix develop`); tasks are in the `justfile`.
- `crates/protocol` is the contract between the server and every client.
  After changing it, run `just codegen`: the regenerated
  `packages/protocol/src/generated` is checked in (CI verifies it is current).
- Database changes go in a new migration (`just migration <name>`), never by
  editing an applied one.
- Before finishing: `just check` and `just test`.
