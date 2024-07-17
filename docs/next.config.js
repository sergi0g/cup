const withNextra = require("nextra")({
  theme: "nextra-theme-docs",
  themeConfig: "./theme.config.jsx",
});

module.exports = withNextra(
  {
    output: "export",
    images: {
      unoptimized: true
    },
    basePath: process.env.NODE_ENV == 'production' ? '/cup' : ''
  }
);

// If you have other Next.js configurations, you can pass them as the parameter:
// module.exports = withNextra({ /* other next.js config */ })
