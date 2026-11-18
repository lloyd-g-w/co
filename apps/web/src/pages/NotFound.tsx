import { paths } from "../router.ts";

export default function NotFound() {
  return (
    <section>
      <h1>Not found</h1>
      <a href={paths()}>Home</a>
    </section>
  );
}
