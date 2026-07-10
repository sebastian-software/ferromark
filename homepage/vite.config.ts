import { defineConfig } from 'vite'
import { ardo } from 'ardo/vite'

export default defineConfig({
  base: '/ferromark/',
  plugins: [
    ardo({
      title: 'ferromark',
      description: 'High-throughput Markdown to HTML parser for Rust',
      githubPages: false,
    }),
  ],
})
