/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "src/static/template.liquid"
  ],
  theme: {
    extend: {},
  },
  plugins: [],
  safelist: [
    {
      pattern: /(bg|text|divide)-(gray|neutral)-.+/,
      variants: ["dark"]
    }
  ]
}

