import type { LinksFunction, MetaFunction } from "react-router";

import {
  ArdoErrorBoundary,
  ArdoGeneratedSidebar,
  ArdoRoot,
  ArdoRootLayout,
  ArdoSearch,
  ArdoSidebar,
  ArdoSidebarGroup,
  ArdoSidebarLink,
  ArdoSidebarSection,
  ArdoThemeToggle,
} from "ardo/ui";
import { MarkDefs, SiteFooter, SiteHeader, SiteMenu } from "ferramenta-family";
import displayFont from "ferramenta-family/fonts/barlow-condensed-700.woff2?url";
import { NavLink, useLocation } from "react-router";
import config from "virtual:ardo/config";

import { documentationSections, sharedConcepts } from "./navigation";
import { version } from "./version";
import "ardo/ui/styles.css";
import "ferramenta-family/tokens.css";
import "ferramenta-family/fonts.css";
import "ferramenta-family/theme.css";
import "ferramenta-family/landing.css";
import "ferramenta-family/docs.css";

import "./styles/site.css";
// Last on purpose, as the package README requires: the shared chrome has to win
// the selector ties the site stylesheet would otherwise take.
import "ferramenta-family/chrome.css";

export const links: LinksFunction = () => [
  {
    rel: "preload",
    href: displayFont,
    as: "font",
    type: "font/woff2",
    crossOrigin: "anonymous",
  },
];

export const meta: MetaFunction = ({ location }) => {
  const path = location.pathname.replace(/\/$/, "");
  const section = documentationSections.find((candidate) => path.startsWith(`/${candidate.id}/`));
  const page = section?.pages.find(([, to]) => to === path);
  const title = page
    ? `${page[0]} · ${section?.label} · Ferromark`
    : "Ferromark — Markdown at native speed";
  return [
    { title },
    {
      name: "description",
      content:
        "A native Markdown engine for Rust and Node.js. Compose typography, GitHub references, and emoji transforms, add publishing features, and explore reproducible benchmarks.",
    },
  ];
};

export function Layout({ children }: { children: React.ReactNode }) {
  return <ArdoRootLayout iconBasePath="/">{children}</ArdoRootLayout>;
}

export const ErrorBoundary = ArdoErrorBoundary;

/*
 * The site's own header and the family footer replace Ardo's chrome, so Ardo
 * must not render either: `chrome` is read from every route match and no route
 * below this one overrides it. The sidebar remains enabled for documentation
 * routes.
 */
export const handle = { chrome: false };

function GuideNav() {
  const { pathname } = useLocation();
  return (
    <nav className="site-links" aria-label="Documentation">
      {documentationSections.map((section) => (
        <NavLink
          key={section.id}
          to={section.to}
          data-active={pathname.startsWith(`/${section.id}/`)}
        >
          {section.label}
        </NavLink>
      ))}
    </nav>
  );
}

function GuideMenuLinks({
  current,
}: {
  current: (typeof documentationSections)[number] | undefined;
}) {
  return (
    <nav className="ferromark-guide-flyout" aria-label="Mobile documentation">
      <NavLink to="/">ferromark home</NavLink>
      <div className="ferromark-section-switch" aria-label="Documentation sections">
        {documentationSections.map((section) => (
          <NavLink key={section.id} to={section.to} data-active={current?.id === section.id}>
            {section.label}
          </NavLink>
        ))}
      </div>
      {current && (
        <>
          <p className="ferromark-menu-label">{current.label}</p>
          {current.pages.map(([label, to]) => (
            <NavLink key={to} to={to}>
              {label}
            </NavLink>
          ))}
        </>
      )}
      {current?.id !== "guide" && (
        <>
          <p className="ferromark-menu-label">Shared concepts</p>
          {sharedConcepts.map(([label, to]) => (
            <NavLink key={to} to={to}>
              {label}
            </NavLink>
          ))}
        </>
      )}
    </nav>
  );
}

function DocsActions() {
  const { pathname } = useLocation();
  const current = documentationSections.find((section) => pathname.startsWith(`/${section.id}/`));
  return (
    <>
      <div className="site-search">
        <ArdoSearch />
      </div>
      <SiteMenu label="Docs">
        <GuideMenuLinks current={current} />
      </SiteMenu>
    </>
  );
}

function FamilyFooter() {
  return (
    <SiteFooter
      current="ferromark"
      legal={
        <>
          {`Ferromark v${version}`} · Released under the MIT License ·{" "}
          <a href="https://ardo-docs.dev">Built with Ardo</a>
        </>
      }
    />
  );
}

export default function Root() {
  const { pathname } = useLocation();
  return (
    <>
      <MarkDefs />
      <SiteHeader
        current="ferromark"
        lockup="project"
        nav={<GuideNav />}
        actions={<DocsActions />}
        themeToggle={pathname === "/" ? undefined : <ArdoThemeToggle />}
      />

      <div className="fam-docs-shell ferromark-shell">
        <ArdoRoot config={config}>
          <ArdoSidebar>
            {documentationSections.map((section) => (
              <ArdoSidebarSection
                key={section.id}
                id={section.id}
                label={section.label}
                to={section.to}
              >
                <ArdoSidebarGroup title={section.label} collapsible={false}>
                  <ArdoGeneratedSidebar section={section.id} />
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
        </ArdoRoot>
      </div>

      <FamilyFooter />
    </>
  );
}
