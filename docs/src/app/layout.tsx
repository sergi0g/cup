import type { Metadata } from "next";
import { Footer, Layout, Navbar, ThemeSwitch } from "nextra-theme-docs";
import { getPageMap } from "nextra/page-map";
import { GeistSans } from "geist/font/sans";
import "nextra-theme-docs/style.css";
import "./globals.css";
import { Head } from "./components/Head";
import Logo from "./components/Logo";

export const metadata: Metadata = {
  title: "Cup",
  description: "The easiest way to manage your container updates",
};

const logo = (
  <div className="flex items-center">
    <Logo />
    <h1 className="ml-2 font-bold">Cup</h1>
  </div>
);

const navbar = (
  <Navbar logo={logo} projectLink="https://github.com/sergi0g/cup" chatLink="https://discord.gg/jmh5ctzwNG">
    <ThemeSwitch lite className="cursor-pointer" />
  </Navbar>
);

const footer = <Footer> </Footer>;

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      dir="ltr"
      suppressHydrationWarning
      className={`${GeistSans.className} antialiased`}
    >
      <Head />
      <body>
        <Layout
          navbar={navbar}
          pageMap={await getPageMap()}
          footer={footer}
          docsRepositoryBase="https://github.com/sergi0g/cup/blob/main/docs"
        >
          <div>{children}</div>
        </Layout>
      </body>
    </html>
  );
}
