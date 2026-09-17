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
import { Mark, MarkDefs, SiteFooter, ToolSwitcher } from "ferramenta-family";
import bigShouldersFont from "ferramenta-family/fonts/big-shoulders.woff2?url";
import { useRef } from "react";
import { NavLink, useLocation } from "react-router";
import config from "virtual:ardo/config";

import { documentationSections, sharedConcepts } from "./navigation";
import "ardo/ui/styles.css";
import "ferramenta-family/tokens.css";
import "ferramenta-family/fonts.css";
import "ferramenta-family/theme.css";

import "./styles/site.css";
// Last on purpose, as the package README requires: the shared chrome has to win
// the selector ties the site stylesheet would otherwise take.
import "ferramenta-family/chrome.css";

export const links: LinksFunction = () => [
  {
    rel: "preload",
    href: bigShouldersFont,
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
    : "Ferromark — Markdown, ready for your pipeline";
  return [
    { title },
    {
      name: "description",
      content:
        "A focused Markdown-to-HTML engine. Dedicated Rust and Node.js guides, shared syntax and rendering contracts, and reproducible performance evidence.",
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
    <nav className="ferromark-nav" aria-label="Documentation">
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
  close,
}: {
  current: (typeof documentationSections)[number] | undefined;
  close: () => void;
}) {
  return (
    <nav className="ferromark-guide-flyout" aria-label="Mobile documentation">
      <NavLink to="/" onClick={close}>
        ferromark home
      </NavLink>
      <div className="ferromark-section-switch" aria-label="Documentation sections">
        {documentationSections.map((section) => (
          <NavLink
            key={section.id}
            to={section.to}
            onClick={close}
            data-active={current?.id === section.id}
          >
            {section.label}
          </NavLink>
        ))}
      </div>
      {current && (
        <>
          <p className="ferromark-menu-label">{current.label}</p>
          {current.pages.map(([label, to]) => (
            <NavLink key={to} to={to} onClick={close}>
              {label}
            </NavLink>
          ))}
        </>
      )}
      {current?.id !== "guide" && (
        <>
          <p className="ferromark-menu-label">Shared concepts</p>
          {sharedConcepts.map(([label, to]) => (
            <NavLink key={to} to={to} onClick={close}>
              {label}
            </NavLink>
          ))}
        </>
      )}
    </nav>
  );
}

function DocsActions() {
  const menuRef = useRef<HTMLDetailsElement>(null);
  const { pathname } = useLocation();
  const current = documentationSections.find((section) => pathname.startsWith(`/${section.id}/`));
  const close = () => menuRef.current?.removeAttribute("open");
  return (
    <>
      <div className="ferromark-search">
        <ArdoSearch />
      </div>
      <details
        className="ferromark-guide-menu"
        ref={menuRef}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            close();
            menuRef.current?.querySelector("summary")?.focus();
          }
        }}
      >
        <summary aria-label="Documentation menu">Docs</summary>
        <GuideMenuLinks current={current} close={close} />
      </details>
    </>
  );
}

/*
 * The header bar. The family package's `SiteHeader` puts the Ferramenta lockup
 * in the brand slot and the site's own name into the navigation, which reads
 * as the family site with a ferromark section. This is ferromark's site, so
 * the bar is composed here from the package's parts instead: the iron bar and
 * the lockup are the package's classes (`site-header`, `wrap`, `bar`,
 * `lockup`, `ghlink`, styled by chrome.css), ferromark takes the lockup, and
 * the family becomes one quiet control at the end of the bar — the package's
 * `ToolSwitcher` with a small "Ferramenta" trigger, so the other tools stay one
 * click away without the family outranking the product. The footer keeps the
 * full family chrome.
 */
function SiteBar() {
  return (
    <header className="site-header">
      <div className="wrap bar">
        <NavLink to="/" className="lockup ferromark-lockup">
          <Mark name="ferromark" size={26} />
          <span>ferromark</span>
        </NavLink>
        <GuideNav />
        <nav className="site" aria-label="Site">
          <ToolSwitcher
            current="ferromark"
            className="ferromark-family"
            label={
              <>
                <Mark name="ferramenta" className="mark ferromark-family-mark" size={18} />
                <span>Ferramenta</span>
              </>
            }
          />
          <a className="ghlink" href="https://github.com/sebastian-software" aria-label="GitHub">
            <svg width="20" height="20" viewBox="0 0 16 16" aria-hidden="true">
              <use href="#i-github" />
            </svg>
          </a>
          <DocsActions />
          <ArdoThemeToggle />
        </nav>
      </div>
    </header>
  );
}

function FamilyFooter() {
  return (
    <SiteFooter
      current="ferromark"
      legal={
        <>
          ferromark and this site are MIT-licensed. Copyright {new Date().getFullYear()} Sebastian
          Software GmbH · <a href="https://ardo-docs.dev">Built with Ardo</a>
        </>
      }
    />
  );
}

export default function Root() {
  return (
    <>
      <MarkDefs />
      <SiteBar />

      {/* `ferromark-shell` is the hook site.css needs to turn Ardo's
          fixed-viewport application shell into a document-scrolling page: the
          family footer sits below the shell, so the page — not the article —
          has to be what scrolls. */}
      <div className="ferromark-shell">
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
