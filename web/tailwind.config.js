/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/App.tsx", "./src/components/*.tsx"],
  theme: {
    extend: {},
  },
  plugins: [],
  safelist: [
    // Generate minimum extra CSS
    {
      pattern: /bg-(gray|neutral)-50/,
    },
    {
      pattern: /bg-(gray|neutral)-(900|950)/,
      variants: ["dark"],
    },
    {
      pattern: /bg-(gray|neutral)-200/,
      variants: ["before", "after"],
    },
    {
      pattern: /bg-(gray|neutral)-800/,
      variants: ["before:dark", "after:dark"],
    },
    {
      pattern: /text-(gray|neutral)-400/,
    },
    {
      pattern: /text-(gray|neutral)-500/,
      variants: ["dark"],
    },
    {
      pattern: /divide-(gray|neutral)-800/,
      variants: ["dark"],
    },
  ],
};
