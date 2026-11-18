import { ConnectionProvider } from "./connection.tsx";
import { paths, Router } from "./router.ts";

export function App() {
  return (
    <ConnectionProvider>
      <Router>
        {(props) => (
          <div class="shell">
            <header>
              <a href={paths()}>co</a>
            </header>
            <main>{props.children}</main>
          </div>
        )}
      </Router>
    </ConnectionProvider>
  );
}
