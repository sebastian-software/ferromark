import { ArdoLink as Link } from "ardo/runtime";
import { FerraChoices } from "ferramenta-family/ferra";

export function PlatformChoices() {
  return (
    <FerraChoices>
      <article>
        <p>Rust crate</p>
        <h2>Build with Rust</h2>
        <p>
          Render HTML, inspect the document tree, and connect your own rendering hooks. Arena
          allocation for content tooling.
        </p>
        <Link to="/rust/getting-started">
          Start with Rust <span aria-hidden="true">→</span>
        </Link>
      </article>
      <article>
        <p>Also available for Node.js</p>
        <h2>Build with Node.js</h2>
        <p>
          A native engine behind a typed JavaScript API. HTML strings, Buffers, metadata, and
          Ferriki highlighting.
        </p>
        <Link to="/node/getting-started">
          Start with Node.js <span aria-hidden="true">→</span>
        </Link>
      </article>
    </FerraChoices>
  );
}
