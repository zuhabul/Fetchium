import type { Config } from 'tailwindcss'

const config: Config = {
  content: ['./src/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    extend: {
      colors: {
        admin: {
          bg: '#09090b',      // zinc-950
          surface: '#18181b', // zinc-900
          border: '#27272a',  // zinc-800
          muted: '#3f3f46',   // zinc-700
          text: '#fafafa',    // zinc-50
          subtle: '#a1a1aa',  // zinc-400
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'ui-monospace', 'monospace'],
      },
    },
  },
  plugins: [],
}

export default config
