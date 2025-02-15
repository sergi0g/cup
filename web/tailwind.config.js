/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/App.tsx", "./src/components/*.tsx", "./index.liquid"],
  theme: {
    extend: {},
  },
  plugins: [require("tailwindcss-animate")],
  safelist: [
    // Generate minimum extra CSS
    {
      pattern: /bg-(gray|neutral)-(50|200|500)/,
    },
    {
      pattern: /bg-(gray|neutral)-100/,
      variants: ["hover"],
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
      variants: ["before:dark", "after:dark", "dark", "hover:dark"],
    },
    {
      pattern: /text-(gray|neutral)-(50|300|200)/,
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
      pattern: /text-(gray|neutral)-600/,
      variants: ["placeholder"],
    },
    {
      pattern: /text-(gray|neutral)-400/,
      variants: ["placeholder:dark"],
    },
    {
      pattern: /text-(gray|neutral)-700/,
    },
    {
      pattern: /text-(gray|neutral)-800/,
      variants: ["group-data-[hover]"],
    },
    {
      pattern: /text-(gray|neutral)-200/,
      variants: ["group-data-[hover]:dark"],
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
