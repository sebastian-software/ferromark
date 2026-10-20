import type { ReactNode } from "ardo/react";
import type { LinksFunction, MetaFunction } from "ardo/router";

import { useRef } from "ardo/react";
import { NavLink, useLocation } from "ardo/router";
import {
  ArdoErrorBoundary,
  ArdoRoot,
  ArdoRootLayout,
  ArdoSearch,
  ArdoSidebar,
  ArdoSidebarGroup,
  ArdoSidebarLink,
  ArdoSidebarSection,
} from "ardo/ui";
import { FerraFooter, FerraHeader, FerraPage, FerraWorkshop } from "ferramenta-family/ferra";
import touch from "ferramenta-family/ferra-assets/ferromark/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferromark/favicon.ico?url";
import mark64 from "ferramenta-family/ferra-assets/ferromark/mark-64.png?url";
import plex from "ferramenta-family/ferra-assets/fonts/ibm-plex-sans-600.woff2?url";
import { family, FAMILY_SITE } from "ferramenta-family/registry";
import config from "virtual:ardo/config";

import { documentationSections, sharedConcepts } from "./navigation";
import "ardo/ui/styles.css";
import "ferramenta-family/ferra.css";

import "./styles/site.css";
import "./styles/docs.css";

const project = family.find((tool) => tool.name === "ferromark");
if (project === undefined) throw new Error("Ferromark is missing from the family registry");
const projectRepo = project.repo;

export const links: LinksFunction = () => [
  { rel: "preload", href: plex, as: "font", type: "font/woff2", crossOrigin: "anonymous" },
];

export const meta: MetaFunction = ({ location }) => {
  const path = location.pathname.replace(/\/$/u, "");
  const section = documentationSections.find((item) => path.startsWith(`/${item.id}/`));
  const page = section?.pages.find(([, to]) => to === path);
  return [
    { title: page ? `${page[0]} · Ferromark` : "Ferromark — Markdown, ready for your pipeline" },
    {
      name: "description",
      content:
        "A Markdown renderer in Rust. Explicit output policies, publishing metadata, and reproducible performance evidence.",
    },
    { name: "color-scheme", content: "light" },
  ];
};

export function Layout({ children }: { children: ReactNode }) {
  return (
    <ArdoRootLayout className="ferra-tokens" theme="light" favicon={mark64} iconBasePath={false}>
      <link rel="icon" href={favicon} sizes="32x32 48x48 64x64" />
      <link rel="apple-touch-icon" href={touch} />
      {children}
    </ArdoRootLayout>
  );
}

export const ErrorBoundary = ArdoErrorBoundary;
export const handle = { chrome: false };

function DocumentationMenu() {
  const ref = useRef<HTMLDetailsElement>(null);
  const { pathname } = useLocation();
  const current = documentationSections.find((item) => pathname.startsWith(`/${item.id}/`));
  const close = () => ref.current?.removeAttribute("open");
  const pages =
    current?.id === "guide" ? current.pages : [...(current?.pages ?? []), ...sharedConcepts];
  return (
    <details
      className="ferromark-guide-menu"
      ref={ref}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          close();
          ref.current?.querySelector("summary")?.focus();
        }
      }}
    >
      <summary>Documentation</summary>
      <nav aria-label="Documentation pages" className="ferromark-guide-flyout">
        {documentationSections.map((section) => (
          <NavLink key={section.id} to={section.to} onClick={close}>
            {section.label}
          </NavLink>
        ))}
        {current && <p>{current.label}</p>}
        {pages.map(([label, to]) => (
          <NavLink key={to} to={to} onClick={close}>
            {label}
          </NavLink>
        ))}
      </nav>
    </details>
  );
}

function documentationSidebar() {
  return (
    <ArdoSidebar>
      {documentationSections.map((section) => (
        <ArdoSidebarSection key={section.id} id={section.id} label={section.label} to={section.to}>
          <ArdoSidebarGroup title={section.label} collapsible={false}>
            {section.pages.map(([label, to]) => (
              <ArdoSidebarLink key={to} to={to}>
                {label}
              </ArdoSidebarLink>
            ))}
          </ArdoSidebarGroup>
          {section.id !== "guide" && (
            <ArdoSidebarGroup title="Shared concepts" collapsible={false}>
              {sharedConcepts.map(([label, to]) => (
                <ArdoSidebarLink key={to} to={to}>
                  {label}
                </ArdoSidebarLink>
              ))}
            </ArdoSidebarGroup>
          )}
        </ArdoSidebarSection>
      ))}
    </ArdoSidebar>
  );
}

function pageClassName(pathname: string) {
  if (pathname === "/") return "ferromark-home";
  const illustrated = [
    "/rust/getting-started",
    "/node/getting-started",
    "/guide/getting-started",
    "/guide/rendering",
    "/guide/benchmarks",
  ].includes(pathname.replace(/\/$/u, ""));
  return `ferromark-docs ferra-wrap${illustrated ? " ferromark-docs-illustrated" : ""}`;
}

export default function Root() {
  const { pathname } = useLocation();
  return (
    <FerraPage className="ferromark-site">
      <a className="ferromark-skip" href="#main-content">
        Skip to content
      </a>
      <FerraHeader
        product="ferromark"
        home="/"
        nav={
          <>
            <NavLink to="/rust/getting-started">Rust</NavLink>
            <NavLink to="/node/getting-started">Node.js</NavLink>
            <NavLink className="ferromark-desktop-link" to="/guide/rendering">
              Rendering
            </NavLink>
            <NavLink className="ferromark-desktop-link" to="/guide/benchmarks">
              Benchmarks
            </NavLink>
            <DocumentationMenu />
            <div className="ferromark-search">
              <ArdoSearch />
            </div>
          </>
        }
      />
      <ArdoRoot config={config} className={pageClassName(pathname)}>
        {documentationSidebar()}
      </ArdoRoot>
      <FerraWorkshop id="workshop-links" />
      <FerraFooter product="ferromark" home="/">
        <a href={FAMILY_SITE}>The Ferramenta family</a>
        <a href={projectRepo}>Ferromark on GitHub</a>
      </FerraFooter>
    </FerraPage>
  );
}
