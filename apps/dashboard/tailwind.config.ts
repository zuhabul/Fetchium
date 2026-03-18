import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        brand: {
          50:  "#f0f4ff",
          100: "#e0eaff",
          200: "#c7d7fe",
          300: "#a5bcfc",
          400: "#8098f9",
          500: "#6172f3",
          600: "#4e53e8",
          700: "#3f43cf",
          800: "#3438a8",
          900: "#2f3585",
          950: "#1c1f51",
        },
        surface: {
          DEFAULT: "#0a0b0f",
          1: "#111318",
          2: "#1a1d24",
          3: "#23272f",
        },
      },
    },
  },
  plugins: [],
} satisfies Config;
