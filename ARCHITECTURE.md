# co architecture

co is a coding agent harness: the server owns the agents, sessions and users;
frontends only render and take input; tool hosts run tools on the machines
where the code lives. A session can switch hosts at any time.

```
 terminal ── co (TUI, ratatui) ──────────┐
 browser ─── web (SolidJS) ──────────────┤  JSON over WebSocket (/ws)
 desktop ─── Tauri (web UI + tool host) ─┤
 machine ─── co tool-host ───────────────┴─▶ co serve (axum)
                                                ├─ store: Postgres (users, argon2 hashes, settings, hosts)
                                                ├─ hosts: registry of connected tool hosts (by host id)
                                                └─ agent (one per session) ── providers ── model APIs
                                                         └─ tool calls ──▶ the session's active host
```

## Layout

| Path | What |
|---|---|
| `crates/protocol` | Wire types (serde). Single source of truth; exported to TypeScript with ts-rs (`just codegen`). |
| `crates/store` | Postgres via sqlx: users, password hashes, settings, hosts. Migrations in `crates/store/migrations`. |
| `crates/hosts` | Host registry: host identity, routing calls to hosts, in-flight calls, reconnect handover. |
| `crates/agent` | Agent loop, session location (active host, pinning, cwd per host). |
| `crates/providers` | Model provider clients (streaming). |
| `crates/tools` | Tool definitions and their implementations (run on hosts). |
| `crates/server` | HTTP/WebSocket server: auth, clients, sessions, hosts. |
| `crates/client` | Rust client SDK (TUI, tool host). |
| `crates/tool-host` | Connects a machine as a tool host; used by `co tool-host` and the desktop app. |
| `apps/cli` | The `co` binary: `serve`, `tool-host`, and the TUI (default). |
| `apps/tui` | The ratatui frontend. |
| `apps/web` | SolidJS frontend (Vite+). |
| `apps/desktop` | Tauri app: the web frontend in a native window plus an in-process tool host. |
| `packages/protocol` | Generated TypeScript protocol types. |
| `packages/client` | TypeScript WebSocket client. |

## Protocol

One JSON message per WebSocket text frame. A client sends `hello` first
(name, kind, whether it runs tools, its host id, its token); the server
answers `hello_reply`. Then requests `{type: "request", id, method, params}`
are answered by `{type: "response", id, ok, result | error}`, and the server
pushes `{type: "event", event, data}`.

## Hosts and switching mid-session

- The agent and transcript live in the server; hosts only execute tools.
- A machine keeps a persistent host id in `~/.co/host-id`, so reconnecting or
  restarting is the same host. A newer connection with the same id takes it over.
- A session has an active host (pinned by the user, or a default) and
  remembers its cwd on every host it has used; switching hosts restores it.
- Host switches are recorded in the session, so a reloaded session resumes on
  its host (waiting for it if it is offline).
- Tool calls may name another host for one call without moving the session.
- In-flight calls finish on the host they started on.
