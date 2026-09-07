import { useRef } from "react"
import {
  ArdoErrorBoundary,
  ArdoGeneratedSidebar,
  ArdoRoot,
  ArdoRootLayout,
  ArdoSearch,
  ArdoSidebar,
  ArdoSidebarSection,
  ArdoThemeToggle,
} from "ardo/ui"
import { MarkDefs, SiteFooter, SiteHeader } from "@ferramenta/family"
import bigShouldersFont from "@ferramenta/family/fonts/big-shoulders.woff2?url"
import config from "virtual:ardo/config"
import { NavLink } from "react-router"
import type { LinksFunction, MetaFunction } from "react-router"
import "ardo/ui/styles.css"
import "@ferramenta/family/tokens.css"
import "@ferramenta/family/fonts.css"
import "@ferramenta/family/theme.css"
import "./styles/site.css"
// Last on purpose, as the package README requires: the shared chrome has to win
// the selector ties the site stylesheet would otherwise take.
import "@ferramenta/family/chrome.css"

export const links: LinksFunction = () => [
  {
    rel: "preload",
    href: bigShouldersFont,
    as: "font",
    type: "font/woff2",
    crossOrigin: "anonymous",
  },
]

export const meta: MetaFunction = () => [
  { title: "ferromark — high-throughput Markdown for Rust and Node.js" },
  {
    name: "description",
    content: "A streaming CommonMark and GFM compiler for high-throughput Rust and Node.js pipelines.",
  },
]

export function Layout({ children }: { children: React.ReactNode }) {
  return <ArdoRootLayout iconBasePath="/ferromark/">{children}</ArdoRootLayout>
}

export const ErrorBoundary = ArdoErrorBoundary

/*
 * The family chrome replaces Ardo's own header and footer, so Ardo must not
 * render either: `chrome` is read from every route match and no route below
 * this one overrides it. The sidebar is not part of that switch — the guide
 * rail and its generated navigation stay exactly as they were.
 */
export const handle = { chrome: false }

/*
 * The guide, in reading order. One list feeds both header affordances: the menu
 * shows all of it, the bar the four links the header carried before — the bar
 * has the tool switcher and the search field to fit as well.
 */
const guidePages = [
  { label: "Getting Started", to: "/guide/getting-started", inBar: false },
  { label: "Quick Start", to: "/guide/quick-start", inBar: true },
  { label: "Features", to: "/guide/features", inBar: true },
  { label: "Benchmarks", to: "/guide/benchmarks", inBar: true },
  { label: "MDX", to: "/guide/mdx-examples", inBar: true },
]

/*
 * The family header carries one slot (`themeToggle`, at the end of the bar), so
 * the documentation affordances Ardo's own header used to provide ride in it:
 * the guide links, full-text search, and — below the breakpoint where Ardo
 * hides the sidebar rail — a menu holding the same links, which is the only way
 * into the guide on a phone. `ArdoSearch` reads its index from a virtual module
 * and falls back to the default labels, so it works outside `ArdoRoot`.
 */
function DocsTools() {
  const menuRef = useRef<HTMLDetailsElement>(null)

  return (
    <>
      <div className="ferromark-nav">
        {guidePages
          .filter((page) => page.inBar)
          .map((page) => (
            <NavLink key={page.to} to={page.to}>
              {page.label}
            </NavLink>
          ))}
      </div>

      <div className="ferromark-search">
        <ArdoSearch />
      </div>

      <details className="ferromark-guide-menu" ref={menuRef}>
        <summary aria-label="Guide pages">Guide</summary>
        <div className="ferromark-guide-flyout">
          {guidePages.map((page) => (
            <NavLink
              key={page.to}
              to={page.to}
              /* Client-side navigation keeps the page mounted, so the menu has
                 to close itself when one of its links is taken. */
              onClick={() => menuRef.current?.removeAttribute("open")}
            >
              {page.label}
            </NavLink>
          ))}
        </div>
      </details>

      <ArdoThemeToggle />
    </>
  )
}

export default function Root() {
  return (
    <>
      <MarkDefs />
      <SiteHeader current="ferromark" themeToggle={<DocsTools />} />

      {/* `ferromark-shell` is the hook site.css needs to turn Ardo's
          fixed-viewport application shell into a document-scrolling page: the
          family footer sits below the shell, so the page — not the article —
          has to be what scrolls. */}
      <div className="ferromark-shell">
        <ArdoRoot config={config}>
          <ArdoSidebar>
            <ArdoSidebarSection id="guide" label="Guide" to="/guide/getting-started">
              <ArdoGeneratedSidebar section="guide" />
            </ArdoSidebarSection>
          </ArdoSidebar>
        </ArdoRoot>
      </div>

      <SiteFooter
        current="ferromark"
        legal={
          <>
            ferromark is dual-licensed under MIT or Apache-2.0; this site is MIT-licensed. Copyright{" "}
            {new Date().getFullYear()} Sebastian Software GmbH ·{" "}
            <a href="https://ardo-docs.dev">Built with Ardo</a>
          </>
        }
      />
    </>
  )
}
