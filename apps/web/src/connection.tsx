import { type Client, type ConnectionStatus, connect } from "@co/client";
import {
  type Accessor,
  createContext,
  createSignal,
  onSettled,
  type ParentComponent,
  useContext,
} from "solid-js";

const DEFAULT_SERVER_URL = "ws://localhost:7420/ws";

export const serverUrl: string = import.meta.env.VITE_CO_SERVER_URL ?? DEFAULT_SERVER_URL;

interface Connection {
  client: Accessor<Client | null>;
  status: Accessor<ConnectionStatus>;
}

const ConnectionContext = createContext<Connection>();

/** Owns the app's single server connection for as long as it is mounted. */
export const ConnectionProvider: ParentComponent = (props) => {
  const [status, setStatus] = createSignal<ConnectionStatus>({ state: "connecting", attempt: 0 });
  const [client, setClient] = createSignal<Client | null>(null);

  onSettled(() => {
    const c = connect(serverUrl, { name: "co-web", client: "web", onStatus: setStatus });
    setClient(c);
    return () => c.close();
  });

  return <ConnectionContext value={{ client, status }}>{props.children}</ConnectionContext>;
};

/** Throws ContextNotFoundError outside <ConnectionProvider>. */
export const useConnection = (): Connection => useContext(ConnectionContext);
