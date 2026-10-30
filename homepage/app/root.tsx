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
  FerraFooter,
  FerraGuideNavigation,
  FerraHeader,
  FerraPage,
  FerraReadingLayout,
  FerraTabs,
} from "ferramenta-family/ferra";
import touchIcon from "ferramenta-family/ferra-assets/ferromark/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferromark/favicon.ico?url";
import { WORKSHOP } from "ferramenta-family/registry";
import "ferramenta-family/ferra.css";

import { DocumentHeading } from "./components/document-heading";
import { documentationSections, sharedConcepts } from "./navigation";
import { version } from "./version";
import "ferramenta-family/ferra-ardo.css";

export function Layout({ children }: { children: ReactNode }) {
  return (
    <ArdoRootLayout
      colorScheme="light"
      className="ferra-tokens ferra-ardo"
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
    <FerraGuideNavigation aria-label="Documentation pages">
      <NavLink to="/" onClick={onNavigate}>
        Overview
      </NavLink>
      <FerraTabs aria-label="Documentation sections">
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
      </FerraTabs>
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
    </FerraGuideNavigation>
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
    <FerraReadingLayout
      id="ferromark-article"
      navigation={<GuideNavigation />}
      outline={<ArdoTOC source="dom" scope="#ferromark-article" />}
    >
      <header>
        <p>Ferromark / {current?.label ?? "Documentation"}</p>
        <h1>{page?.title ?? "Documentation"}</h1>
      </header>
      {children}
    </FerraReadingLayout>
  );
}
const components = { h2: DocumentHeading, wrapper: Article };

export default function Root() {
  return (
    <FerraPage className="ferra-site">
      <MarkDefs />
      <ArdoRoot
        className="ferra-ardo-shell"
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
