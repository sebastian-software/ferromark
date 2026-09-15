import { Link } from "react-router";

export function PlatformChoices() {
  return (
    <div className="platform-choices" aria-label="Choose your runtime">
      <article className="platform-primary">
        <p className="platform-label">Rust crate</p>
        <h2>Build with Rust</h2>
        <p>
          Render HTML, inspect the document tree, and connect your own rendering hooks. Arena
          allocation for content tooling.
        </p>
        <Link className="platform-link" to="/rust/getting-started">
          Start with Rust <span aria-hidden="true">→</span>
        </Link>
      </article>
      <article>
        <p className="platform-label">Also available for Node.js</p>
        <h2>Build with Node.js</h2>
        <p>
          A native engine behind a typed JavaScript API. HTML strings, Buffers, metadata, and
          Ferriki highlighting.
        </p>
        <Link className="platform-link" to="/node/getting-started">
          Start with Node.js <span aria-hidden="true">→</span>
        </Link>
      </article>
    </div>
  );
}
