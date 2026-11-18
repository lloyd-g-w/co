import { createRouter } from "@solidjs/router";
import { lazy } from "solid-js";

export const Router = createRouter({
  routes: [
    { path: "/", component: lazy(() => import("./pages/Home.tsx")) },
    { path: "*404", component: lazy(() => import("./pages/NotFound.tsx")) },
  ],
});

export const { paths } = Router;
