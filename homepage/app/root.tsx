import {
  ArdoErrorBoundary,
  ArdoFooter,
  ArdoGeneratedSidebar,
  ArdoHeader,
  ArdoNav,
  ArdoNavLink,
  ArdoRoot,
  ArdoRootLayout,
  ArdoSidebar,
  ArdoSidebarSection,
} from "ardo/ui"
import { FamilyLinks } from "@ferramenta/ardo-config"
import config from "virtual:ardo/config"
import type { MetaFunction } from "react-router"
import "ardo/ui/styles.css"
import "@ferramenta/ardo-config/theme.css"

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

export default function Root() {
  return (
    <ArdoRoot config={config}>
      <ArdoHeader title="ferromark">
        <ArdoNav>
          <ArdoNavLink to="/guide/quick-start">Quick Start</ArdoNavLink>
          <ArdoNavLink to="/guide/benchmarks">Benchmarks</ArdoNavLink>
          <ArdoNavLink to="/guide/features">Features</ArdoNavLink>
          <ArdoNavLink to="/guide/mdx-examples">MDX</ArdoNavLink>
        </ArdoNav>
      </ArdoHeader>

      <ArdoSidebar>
        <ArdoSidebarSection id="guide" label="Guide" to="/guide/getting-started">
          <ArdoGeneratedSidebar section="guide" />
        </ArdoSidebarSection>
      </ArdoSidebar>

      <ArdoFooter>
        <FamilyLinks current="ferromark" />
        <p>
          Copyright {new Date().getFullYear()} Sebastian Software GmbH ·{" "}
          <a href="https://ardo-docs.dev">Built with Ardo</a>
        </p>
      </ArdoFooter>
    </ArdoRoot>
  )
}
