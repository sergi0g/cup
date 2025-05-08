/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/App.tsx", "./src/components/**/*.tsx", "./index.liquid"],
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
      pattern: /bg-(gray|neutral)-(400|900|950)/,
      variants: ["dark"],
    },
    {
      pattern: /bg-(gray|neutral)-200/,
      variants: ["before", "after"],
    },
    {
      pattern: /bg-(gray|neutral)-900/,
      variants: ["before:dark", "after:dark", "dark", "hover:dark"],
    },
    {
      pattern: /text-(gray|neutral)-(50|300|200|400)/,
      variants: ["dark"],
    },
    {
      pattern: /text-(gray|neutral)-600/,
      variants: ["*", "dark", "hover", "placeholder"],
    },
    {
      pattern: /text-(gray|neutral)-400/,
      variants: ["*:dark", "dark", "dark:hover", "placeholder:dark"],
    },
    {
      pattern: /text-(gray|neutral)-700/,
    },
    {
      pattern: /text-(gray|neutral)-950/,
      variants: ["dark:group-data-[state=checked]"]
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
      pattern: /divide-(gray|neutral)-900/,
      variants: ["dark"],
    },
    {
      pattern: /border-(gray|neutral)-(600|300|400)/,
    },
    {
      pattern: /border-(gray|neutral)-(400|700|800|900)/,
      variants: ["dark"],
    },
    {
      pattern: /ring-(gray|neutral)-700/,
    },
    {
      pattern: /ring-(gray|neutral)-400/,
      variants: ["dark"],
    },
  ],
};
