import { ArdoActiveLink as NavLink } from "ardo/runtime";
import { FerraTabs } from "ferramenta-family/ferra";

type Topic = "configuration" | "getting-started" | "highlighting" | "pipelines";

/** Each runtime keeps a linkable page; switching preserves the current topic. */
export function RuntimeSwitch({ topic }: { topic: Topic }) {
  return (
    <FerraTabs aria-label="Runtime for this guide">
      <NavLink to={`/rust/${topic}`}>Rust</NavLink>
      <NavLink to={`/node/${topic}`}>Node.js</NavLink>
    </FerraTabs>
  );
}
