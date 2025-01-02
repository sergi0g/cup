/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/App.tsx", "./src/components/*.tsx"],
  theme: {
    extend: {},
  },
  plugins: [require("tailwindcss-animate")],
  safelist: [
    // Generate minimum extra CSS
    {
      pattern: /bg-(gray|neutral)-(50|100|200|500)/,
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
      variants: ["before:dark", "after:dark", "dark"],
    },
    {
      pattern: /text-(gray|neutral)-(50|300)/,
      variants: ["dark"],
    },
    {
      pattern: /text-(gray|neutral)-600/,
      variants: ["dark", "hover"],
    },
    {
      pattern: /text-(gray|neutral)-400/,
      variants: ["dark", "dark:hover"],
    },
    {
      pattern: /text-(gray|neutral)-500/,
      variants: ["placeholder"],
    },
    {
      pattern: /text-(gray|neutral)-700/,
    },
    {
      pattern: /divide-(gray|neutral)-800/,
      variants: ["dark"],
    },
    {
      pattern: /border-(gray|neutral)-(200|300)/,
    },
    {
      pattern: /border-(gray|neutral)-(700|800)/,
      variants: ["dark"],
    },
  ],
};
