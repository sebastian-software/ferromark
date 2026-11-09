import type { ReactNode } from "ardo/react";

import { ArdoActiveLink as NavLink } from "ardo/runtime";
import {
  MonoArdoErrorBoundary,
  MonoArdoLayout,
  MonoArdoSite,
  type MonoDocsGroup,
  type MonoDocsSection,
} from "ferramenta-family/ardo";
import touchIcon from "ferramenta-family/ferra-assets/ferromark/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferromark/favicon.ico?url";

import { DocumentHeading } from "./components/document-heading";
import { documentationSections, sharedConcepts } from "./navigation";
import { version } from "./version";
import "ferramenta-family/monografie.css";
import "ferramenta-family/monografie-docs.css";
import "ferramenta-family/monografie-ardo.css";

// Ferromark's own specimens, all under fm- classes; after the package so they win its ties.
import "./components/homepage-specimens.css";

export function Layout({ children }: { children: ReactNode }) {
  return (
    <MonoArdoLayout icon={favicon} appleTouchIcon={touchIcon}>
      {children}
    </MonoArdoLayout>
  );
}
export const ErrorBoundary = MonoArdoErrorBoundary;

const sections: MonoDocsSection[] = documentationSections.map((section) => ({
  id: section.id,
  label: section.label,
  to: section.to,
  icon: section.icon,
  pages: section.pages.map(([label, to]) => ({ label, to })),
}));

const groups: MonoDocsGroup[] = [
  {
    title: "Shared concepts",
    pages: sharedConcepts.map(([label, to]) => ({ label, to })),
    hiddenIn: ["guide"],
  },
];

const links = [
  <NavLink key="guide" to="/guide/getting-started">
    Guide
  </NavLink>,
  <NavLink key="rust" to="/rust/getting-started">
    Rust
  </NavLink>,
  <NavLink key="node" to="/node/getting-started">
    Node.js
  </NavLink>,
];

/** Every page on Monografie's Ardo shell; the documentation adds its menu, search and reading layout. */
export default function Root() {
  return (
    <MonoArdoSite
      product="ferromark"
      name="Ferromark"
      release={version}
      links={links}
      repository="https://github.com/sebastian-software/ferromark"
      sections={sections}
      groups={groups}
      components={{ h2: DocumentHeading }}
      homeClassName="fm-site"
      docsClassName="fm-docs"
      footer={
        <>
          <a href="https://github.com/sebastian-software/ferromark">GitHub</a>
          <a href="https://crates.io/crates/ferromark">crates.io</a>
          <a href="https://www.npmjs.com/package/ferromark">npm</a>
        </>
      }
    />
  );
}
