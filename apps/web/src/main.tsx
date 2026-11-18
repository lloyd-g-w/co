import { render } from "@solidjs/web";
import { App } from "./App.tsx";
import "./styles.css";

const root = document.getElementById("app");
if (!root) throw new Error("#app element missing");

render(() => <App />, root);
