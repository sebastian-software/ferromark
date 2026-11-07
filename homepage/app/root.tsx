import { type ReactNode, useState } from "ardo/react";
import { ArdoActiveLink as NavLink, useArdoPageData, useArdoPath } from "ardo/runtime";
import {
  ArdoErrorBoundary,
  ArdoHeaderSearch,
  ArdoMobileNavigation,
  ArdoRoot,
  ArdoRootLayout,
  ArdoTOC,
} from "ardo/ui";
import { Mark, MarkDefs } from "ferramenta-family";
import touchIcon from "ferramenta-family/ferra-assets/ferromark/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferromark/favicon.ico?url";
import {
  MonoFooter,
  MonoGuideNavigation,
  MonoHeader,
  MonoPage,
  MonoReadingLayout,
  MonoReadingTable,
  MonoSectionRail,
} from "ferramenta-family/monografie";

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
    <ArdoRootLayout
      colorScheme="light"
      className="mono-ardo"
      icon={favicon}
      appleTouchIcon={touchIcon}
      iconBasePath={false}
    >
      {children}
    </ArdoRootLayout>
  );
}
export const ErrorBoundary = ArdoErrorBoundary;

function GuideNavigation({ onNavigate }: { onNavigate?: () => void }) {
  const path = useArdoPath();
  const current =
    documentationSections.find((section) => path.startsWith(`/${section.id}/`)) ??
    documentationSections[0];
  return (
    <>
      <MonoSectionRail aria-label="Documentation sections">
        {documentationSections.map((section) => (
          <NavLink
            key={section.id}
            to={section.to}
            data-active={current.id === section.id}
            onClick={onNavigate}
          >
            <Mark name={section.icon} />
            {section.label}
          </NavLink>
        ))}
      </MonoSectionRail>
      <MonoGuideNavigation aria-label="Documentation pages">
        <NavLink to="/" onClick={onNavigate}>
          Overview
        </NavLink>
        <p>{current.label}</p>
        {current.pages.map(([label, to]) => (
          <NavLink key={to} to={to} onClick={onNavigate}>
            {label}
          </NavLink>
        ))}
        {current.id !== "guide" && (
          <>
            <p>Shared concepts</p>
            {sharedConcepts.map(([label, to]) => (
              <NavLink key={to} to={to} onClick={onNavigate}>
                {label}
              </NavLink>
            ))}
          </>
        )}
      </MonoGuideNavigation>
    </>
  );
}

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
  <a key="speed" href="/#speed">
    Benchmarks
  </a>,
  <a key="correctness" href="/#correctness">
    Correctness
  </a>,
  <a key="github" href="https://github.com/sebastian-software/ferromark">
    GitHub
  </a>,
];

/** The guides' section menu and search, beside the header's own menu. */
function DocsActions() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <ArdoMobileNavigation title="Ferromark documentation" open={open} onOpenChange={setOpen}>
        <GuideNavigation
          onNavigate={() => {
            setOpen(false);
          }}
        />
      </ArdoMobileNavigation>
      <ArdoHeaderSearch
        variant="icon"
        onActivate={() => {
          setOpen(false);
        }}
      />
    </>
  );
}

/** The Monografie header on every page; the guides add their section menu and search. */
function Header({ docs }: { docs: boolean }) {
  return (
    <MonoHeader
      product="ferromark"
      home="/"
      release={version}
      links={links}
      actions={docs ? <DocsActions /> : undefined}
    />
  );
}

function Article({ children }: { children: ReactNode }) {
  const page = useArdoPageData();
  const path = useArdoPath();
  const current = documentationSections.find((section) => path.startsWith(`/${section.id}/`));
  return (
    <MonoReadingLayout
      id="ferromark-article"
      navigation={<GuideNavigation />}
      outline={<ArdoTOC source="dom" scope="#ferromark-article" />}
    >
      <header>
        <p>Ferromark / {current?.label ?? "Documentation"}</p>
        <h1>{page?.title ?? "Documentation"}</h1>
      </header>
      {children}
    </MonoReadingLayout>
  );
}
const components = { h2: DocumentHeading, table: MonoReadingTable, wrapper: Article };

export default function Root() {
  const home = useArdoPath() === "/";
  const shell = (
    <>
      <ArdoRoot
        className="mono-ardo-shell"
        scrollMode="document"
        header={<Header docs={!home} />}
        sidebar={false}
        footer={false}
        components={components}
      />
      <MonoFooter product="ferromark" home="/">
        <a href="https://github.com/sebastian-software/ferromark">GitHub</a>
        <a href="https://crates.io/crates/ferromark">crates.io</a>
        <a href="https://www.npmjs.com/package/ferromark">npm</a>
      </MonoFooter>
    </>
  );
  // The homepage and the guides stand on Monografie's paper; the guides add the reading layout.
  if (home) return <MonoPage className="fm-site">{shell}</MonoPage>;
  return (
    <MonoPage className="mono-docs fm-docs">
      <MarkDefs />
      {shell}
    </MonoPage>
  );
}
