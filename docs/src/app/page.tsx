import { useMDXComponents } from "@/mdx-components";
import { Heading, NextraMetadata } from "nextra";
import Home from "./components/pages/home";

const Wrapper = useMDXComponents({}).wrapper;

const toc: Heading[] = [];

export const metadata: NextraMetadata = {
  title: "Cup - The easiest way to manage your container updates",
  description: "Simple, fast, efficient Docker image update checking",
  filePath: "",
};

export default function Page() {
  return (
    // @ts-ignore
    <Wrapper toc={toc} metadata={metadata} className={"x:mx-auto x:flex"}>
      <Home />
    </Wrapper>
  );
}
