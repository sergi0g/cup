import React from "react";
import Document, { Html, Head, Main, NextScript } from "next/document";
import type { DocumentInitialProps, DocumentContext } from "next/document";

class DocumentProMax extends Document {
  static async getInitialProps(
    ctx: DocumentContext,
  ): Promise<DocumentInitialProps> {
    const initialProps = await Document.getInitialProps(ctx);

    return initialProps;
  }

  render() {
    return (
      <Html lang="en">
        <Head>
          <link
            rel="apple-touch-icon"
            sizes="180x180"
            href="/apple-touch-icon.png"
          />
          <link rel="icon" type="image/svg+xml" href="favicon.svg" />
          <link rel="icon" type="image/x-icon" href="favicon.ico" />
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
        </Head>
        <body>
          <Main />
          <NextScript />
        </body>
      </Html>
    );
  }
}

export default DocumentProMax;
