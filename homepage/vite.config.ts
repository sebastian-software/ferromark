import { defineConfig } from 'vite'
import { ardo } from 'ardo/vite'

export default defineConfig({
  plugins: [
    ardo({
      title: 'ferromark',
      description: 'High-throughput Markdown to HTML parser for Rust',

      typedoc: false,

      // GitHub Pages: base path auto-detected from git remote

      themeConfig: {
        siteTitle: 'ferromark',

        nav: [
          { text: 'Quick Start', link: '/guide/quick-start' },
          { text: 'Benchmarks', link: '/guide/benchmarks' },
          { text: 'Features', link: '/guide/features' },
          { text: 'MDX', link: '/guide/mdx-examples' },
        ],

        sidebar: [
          {
            text: 'Guide',
            items: [
              { text: 'Getting Started', link: '/guide/getting-started' },
              { text: 'Quick Start', link: '/guide/quick-start' },
              { text: 'Benchmarks', link: '/guide/benchmarks' },
              { text: 'Features and Flags', link: '/guide/features' },
              { text: 'MDX Examples', link: '/guide/mdx-examples' },
            ],
          },
        ],

        footer: {
          message: 'ferromark docs and homepage',
        },

        search: {
          enabled: true,
        },
      },
    }),
  ],
})
