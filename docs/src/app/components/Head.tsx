"use client";

import { Head as NextraHead } from "nextra/components";

export function Head() {
  return (
    <NextraHead>
      <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
      <link rel="icon" type="image/x-icon" href="/favicon.ico" />
      <meta
        name="theme-color"
        media="(prefers-color-scheme: light)"
        content="#ffffff"
      />
      <meta
        name="theme-color"
        media="(prefers-color-scheme: dark)"
        content="#111111"
      />
      <meta
        name="og:image"
        content="https://raw.githubusercontent.com/sergi0g/cup/main/docs/public/cup-og.png"
      />
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:site" content="https://cup.sergi0g.dev" />
      <meta name="apple-mobile-web-app-title" content="Cup" />
    </NextraHead>
  );
}
