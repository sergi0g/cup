import { ThemeSwitch } from "nextra-theme-docs";
import { useRouter } from "next/router";
import { useConfig } from "nextra-theme-docs";

export default {
  docsRepositoryBase: "https://github.com/sergi0g/cup/tree/main/docs",
  useNextSeoProps() {
    const { asPath } = useRouter();
    if (asPath !== "/") {
      return {
        titleTemplate: "Cup – %s",
      };
    }
  },
  head: () => {
    const { asPath } = useRouter();
    const { frontMatter } = useConfig();
    const url = "https://sergi0g.github.io/cup/docs/" + `/${asPath}`;

    return (
      <>
        <meta property="og:url" content={url} />
        <meta property="og:title" content={frontMatter.title || "Cup"} />
        <meta
          property="og:description"
          content={
            frontMatter.description ||
            "The easiest way to manage your container updates"
          }
        />
      </>
    );
  },
  logo: (
    <div className="flex items-center">
      <Logo />
      <h1 className="ml-2 font-bold">Cup</h1>
    </div>
  ),
  logoLink: "/",
  project: {
    link: "https://github.com/sergi0g/cup/",
  },
  navbar: {
    extraContent: <ThemeSwitch lite className="[&_span]:hidden" />,
  },
  toc: {
    backToTop: true,
  },
  footer: {
    text: null,
  },
  navigation: false,
};

function Logo() {
  return (
    <svg
      viewBox="0 0 128 128"
      style={{ height: "calc(var(--nextra-navbar-height) * 0.6)" }}
    >
      <path
        style={{ fill: "#A6CFD6" }}
        d="M65.12,17.55c-17.6-0.53-34.75,5.6-34.83,14.36c-0.04,5.2,1.37,18.6,3.62,48.68s2.25,33.58,3.5,34.95
                c1.25,1.37,10.02,8.8,25.75,8.8s25.93-6.43,26.93-8.05c0.48-0.78,1.83-17.89,3.5-37.07c1.81-20.84,3.91-43.9,3.99-45.06
                C97.82,30.66,94.2,18.43,65.12,17.55z"
      />
      <path
        style={{ fill: "#DCEDF6" }}
        d="M41.4,45.29c-0.12,0.62,1.23,24.16,2.32,27.94c1.99,6.92,9.29,7.38,10.23,4.16
                c0.9-3.07-0.38-29.29-0.38-29.29s-3.66-0.3-6.43-0.84C44,46.63,41.4,45.29,41.4,45.29z"
      />
      <path
        style={{ fill: "#6CA4AE" }}
        d="M33.74,32.61c-0.26,8.83,20.02,12.28,30.19,12.22c13.56-0.09,29.48-4.29,29.8-11.7
                S79.53,21.1,63.35,21.1C49.6,21.1,33.96,25.19,33.74,32.61z"
      />
      <path
        style={{ fill: "#DC0D27" }}
        d="M84.85,13.1c-0.58,0.64-9.67,30.75-9.67,30.75s2.01-0.33,4-0.79c2.63-0.61,3.76-1.06,3.76-1.06
                s7.19-22.19,7.64-23.09c0.45-0.9,21.61-7.61,22.31-7.93c0.7-0.32,1.39-0.4,1.46-0.78c0.06-0.38-2.34-6.73-3.11-6.73
                C110.47,3.47,86.08,11.74,84.85,13.1z"
      />
      <path
        style={{ fill: "#8A1F0F" }}
        d="M110.55,7.79c1.04,2.73,2.8,3.09,3.55,2.77c0.45-0.19,1.25-1.84,0.01-4.47
                c-0.99-2.09-2.17-2.74-2.93-2.61C110.42,3.6,109.69,5.53,110.55,7.79z"
      />
      <g>
        <path
          style={{ fill: "#8A1F0F" }}
          d="M91.94,18.34c-0.22,0-0.44-0.11-0.58-0.3l-3.99-5.77c-0.22-0.32-0.14-0.75,0.18-0.97
                  c0.32-0.22,0.76-0.14,0.97,0.18l3.99,5.77c0.22,0.32,0.14,0.75-0.18,0.97C92.21,18.3,92.07,18.34,91.94,18.34z"
        />
      </g>
      <g>
        <path
          style={{ fill: "#8A1F0F" }}
          d="M90.28,19.43c-0.18,0-0.35-0.07-0.49-0.2l-5.26-5.12c-0.28-0.27-0.28-0.71-0.01-0.99
                  c0.27-0.28,0.71-0.28,0.99-0.01l5.26,5.12c0.28,0.27,0.28,0.71,0.01,0.99C90.64,19.36,90.46,19.43,90.28,19.43z"
        />
      </g>
      <g>
        <path
          style={{ fill: "#8A1F0F" }}
          d="M89.35,21.22c-0.12,0-0.25-0.03-0.36-0.1l-5.6-3.39c-0.33-0.2-0.44-0.63-0.24-0.96
                  c0.2-0.33,0.63-0.44,0.96-0.24l5.6,3.39c0.33,0.2,0.44,0.63,0.24,0.96C89.82,21.1,89.59,21.22,89.35,21.22z"
        />
      </g>
    </svg>
  );
}
