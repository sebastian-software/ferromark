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
import { MarkDefs } from "ferramenta-family";
import {
  FerraDocsLayout,
  FerraFooter,
  FerraHeader,
  FerraMark,
  FerraPage,
} from "ferramenta-family/ferra";
import touchIcon from "ferramenta-family/ferra-assets/ferromark/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferromark/favicon.ico?url";
import { WORKSHOP } from "ferramenta-family/registry";
import "ferramenta-family/ferra.css";

import { documentationSections, sharedConcepts } from "./navigation";
import { version } from "./version";
import "./styles/site.css";

export function Layout({ children }: { children: ReactNode }) {
  return (
    <ArdoRootLayout
      colorScheme="light"
      className="ferra-tokens ferromark-theme"
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
    <nav className="ferromark-navigation" aria-label="Documentation pages">
      <NavLink to="/" onClick={onNavigate}>
        Overview
      </NavLink>
      <div className="ferromark-section-switch">
        {documentationSections.map((section) => (
          <NavLink
            key={section.id}
            to={section.to}
            data-active={current.id === section.id}
            onClick={onNavigate}
          >
            {section.label}
          </NavLink>
        ))}
      </div>
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
    </nav>
  );
}

function Header() {
  const [open, setOpen] = useState(false);
  return (
    <FerraHeader
      product="ferromark"
      home="/"
      nav={
        <>
          <NavLink to="/rust/getting-started">Rust</NavLink>
          <NavLink to="/node/getting-started">Node.js</NavLink>
          <NavLink to="/guide/getting-started">Reference</NavLink>
          <a href="https://ferramenta.dev">Ferramenta</a>
        </>
      }
      actions={
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
      }
    />
  );
}

function Article({ children }: { children: ReactNode }) {
  const page = useArdoPageData();
  const path = useArdoPath();
  const current = documentationSections.find((section) => path.startsWith(`/${section.id}/`));
  return (
    <FerraDocsLayout
      id="ferromark-article"
      navigation={<GuideNavigation />}
      outline={<ArdoTOC source="dom" scope="#ferromark-article" />}
      hero={
        <div className="ferromark-reading ardo-doc-content">
          <div className="ferromark-reading-label">
            <FerraMark product="ferromark" size={48} />
            <span>{current?.label ?? "Documentation"} · Ferromark</span>
          </div>
          <h1>{page?.title ?? "Documentation"}</h1>
          {children}
        </div>
      }
    />
  );
}
const components = { wrapper: Article };

export default function Root() {
  return (
    <FerraPage className="ferromark-site">
      <MarkDefs />
      <ArdoRoot
        className="ferromark-shell"
        scrollMode="document"
        header={<Header />}
        sidebar={false}
        footer={false}
        components={components}
      />
      <FerraFooter product="ferromark" home="/">
        <p>
          Ferromark v{version} · MIT License · <a href="https://ardo-docs.dev">Built with Ardo</a> ·{" "}
          <a href={WORKSHOP.openSource}>{WORKSHOP.company}</a> ·{" "}
          <a href="https://ferramenta.dev">Ferramenta family</a>
        </p>
      </FerraFooter>
    </FerraPage>
  );
}
