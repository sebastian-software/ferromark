import type { Config } from "@react-router/dev/config"
import { withArdoGitHubPages } from "ardo/vite"

const config = {
  ssr: false,
  prerender: true,
} satisfies Config

export default withArdoGitHubPages(config, { basename: "/ferromark/" })
