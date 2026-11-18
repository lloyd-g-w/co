import { Match, Switch } from "solid-js";
import { serverUrl, useConnection } from "../connection.tsx";

export default function Home() {
  const { status } = useConnection();
  return (
    <section>
      <h1>co</h1>
      <p>
        Server: <code>{serverUrl}</code>
      </p>
      <p class="status">
        <Switch>
          <Match when={status().state === "open"}>Connected</Match>
          <Match when={status().state === "connecting"}>Connecting…</Match>
          <Match when={status().state === "closed"}>Disconnected</Match>
        </Switch>
      </p>
    </section>
  );
}
