import type { ClientKind, ClientMessage, ErrorBody, ServerMessage } from "@co/protocol";
import { backoffDelay } from "./backoff.ts";

export type ConnectionStatus =
  | { state: "connecting"; attempt: number }
  | { state: "open"; clientId: string; hostId: string | null }
  | { state: "closed"; retryInMs: number | null };

export type EventHandler = (event: string, data: unknown) => void;

export interface ConnectOptions {
  /** Shown to other clients and in the server's logs. */
  name?: string;
  client?: ClientKind;
  token?: string | null;
  /** Reconnect after the socket drops (default true). */
  reconnect?: boolean;
  onStatus?: (status: ConnectionStatus) => void;
  /** For tests: the WebSocket implementation. */
  WebSocket?: typeof WebSocket;
}

export interface Client {
  readonly status: ConnectionStatus;
  request<T = unknown>(method: string, params?: unknown): Promise<T>;
  /** Subscribe to server events; returns an unsubscribe function. */
  onEvent(handler: EventHandler): () => void;
  close(): void;
}

export class CoRequestError extends Error {
  readonly code: string;
  constructor(error: ErrorBody) {
    super(error.message);
    this.name = "CoRequestError";
    this.code = error.code;
  }
}

interface Pending {
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
}

/**
 * Open a connection to a co server. The client sends `hello` first, then
 * matches responses to requests by id. Requests made while disconnected are
 * queued until the next `hello_reply`; ones in flight when the socket drops
 * are rejected (the server may or may not have run them).
 */
export function connect(url: string, options: ConnectOptions = {}): Client {
  const WS = options.WebSocket ?? globalThis.WebSocket;
  const reconnect = options.reconnect ?? true;
  const handlers = new Set<EventHandler>();
  const pending = new Map<number, Pending>();
  let queue: ClientMessage[] = [];
  let nextId = 1;
  let attempt = 0;
  let socket: WebSocket | null = null;
  let ready = false;
  let closedByUser = false;
  let retryTimer: ReturnType<typeof setTimeout> | null = null;
  let status: ConnectionStatus = { state: "connecting", attempt: 0 };

  const setStatus = (s: ConnectionStatus) => {
    status = s;
    options.onStatus?.(s);
  };

  const send = (msg: ClientMessage) => {
    if (socket && ready) socket.send(JSON.stringify(msg));
    else queue.push(msg);
  };

  const onMessage = (raw: string) => {
    let msg: ServerMessage;
    try {
      msg = JSON.parse(raw) as ServerMessage;
    } catch {
      console.warn("co: ignoring non-JSON message", raw);
      return;
    }
    switch (msg.type) {
      case "hello_reply": {
        ready = true;
        attempt = 0;
        setStatus({ state: "open", clientId: msg.client_id, hostId: msg.host_id ?? null });
        const queued = queue;
        queue = [];
        for (const m of queued) socket?.send(JSON.stringify(m));
        break;
      }
      case "response": {
        const p = pending.get(msg.id);
        if (!p) return;
        pending.delete(msg.id);
        if (msg.ok) p.resolve(msg.result);
        else
          p.reject(new CoRequestError(msg.error ?? { code: "unknown", message: "request failed" }));
        break;
      }
      case "event":
        for (const h of handlers) h(msg.event, msg.data);
        break;
    }
  };

  const open = () => {
    setStatus({ state: "connecting", attempt });
    const ws = new WS(url);
    socket = ws;
    ready = false;
    ws.addEventListener("open", () => {
      const hello: ClientMessage = {
        type: "hello",
        name: options.name ?? "co-web",
        client: options.client ?? "web",
        tools: false,
        host_id: null,
        token: options.token ?? null,
      };
      ws.send(JSON.stringify(hello));
    });
    ws.addEventListener("message", (e) => {
      if (typeof e.data === "string") onMessage(e.data);
    });
    ws.addEventListener("close", () => {
      if (socket !== ws) return;
      socket = null;
      ready = false;
      for (const p of pending.values()) p.reject(new Error("connection closed"));
      pending.clear();
      if (closedByUser || !reconnect) {
        setStatus({ state: "closed", retryInMs: null });
        return;
      }
      const delay = backoffDelay(attempt++);
      setStatus({ state: "closed", retryInMs: delay });
      retryTimer = setTimeout(open, delay);
    });
  };

  open();

  return {
    get status() {
      return status;
    },
    request<T>(method: string, params: unknown = null): Promise<T> {
      const id = nextId++;
      return new Promise<T>((resolve, reject) => {
        pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
        send({ type: "request", id, method, params } as ClientMessage);
      });
    },
    onEvent(handler) {
      handlers.add(handler);
      return () => handlers.delete(handler);
    },
    close() {
      closedByUser = true;
      if (retryTimer) clearTimeout(retryTimer);
      socket?.close();
      for (const p of pending.values()) p.reject(new Error("client closed"));
      pending.clear();
      queue = [];
    },
  };
}
