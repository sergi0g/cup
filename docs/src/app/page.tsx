import { useMDXComponents } from "@/mdx-components";
import { Heading, NextraMetadata } from "nextra";
import Home from "./components/pages/home";

/* eslint-disable-next-line */
const Wrapper = useMDXComponents({}).wrapper;

const toc: Heading[] = [];

export const metadata: NextraMetadata = {
  title: "Cup - The easiest way to manage your container updates",
  description: "Simple, fast, efficient Docker image update checking",
  filePath: "",
};

export default function Page() {
  return (
    // @ts-expect-error This component passes all extra props to the underlying component, but that possibility does not exist in the type declarations. A comment there indicates that passing extra props is intended functionality.
    <Wrapper toc={toc} metadata={metadata} className={"x:mx-auto x:flex"}>
      <Home />
    </Wrapper>
  );
}
