import { ArdoActiveLink as NavLink } from "ardo/runtime";
import { MonoTabs } from "ferramenta-family/monografie";

type Topic = "configuration" | "getting-started" | "highlighting" | "pipelines";

/** Each runtime keeps a linkable page; switching preserves the current topic. */
export function RuntimeSwitch({ topic }: { topic: Topic }) {
  return (
    <MonoTabs aria-label="Runtime for this guide">
      <NavLink to={`/rust/${topic}`}>Rust</NavLink>
      <NavLink to={`/node/${topic}`}>Node.js</NavLink>
    </MonoTabs>
  );
}
