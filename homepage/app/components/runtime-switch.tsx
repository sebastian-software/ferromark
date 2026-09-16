import { NavLink } from "react-router";

type Topic = "configuration" | "getting-started" | "highlighting" | "pipelines";

/** Each runtime keeps a linkable page; switching preserves the current topic. */
export function RuntimeSwitch({ topic }: { topic: Topic }) {
  return (
    <nav className="runtime-switch" aria-label="Runtime for this guide">
      <NavLink to={`/rust/${topic}`}>Rust</NavLink>
      <NavLink to={`/node/${topic}`}>Node.js</NavLink>
    </nav>
  );
}
