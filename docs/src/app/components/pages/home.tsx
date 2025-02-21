import React from "react";
import "./styles.css"

import CopyableCode from "../CopyableCode";
import { Browser } from "../Browser";
import { Card } from "../Card";
import {
  IconAdjustments,
  IconArrowRight,
  IconBolt,
  IconBraces,
  IconDevices,
  IconFeather,
  IconLockCheck,
} from "@tabler/icons-react";
import { GitHubIcon } from "nextra/icons";
import { GridPattern } from "../GridPattern";
import { GradientText } from "../GradientText";
import { Section } from "../Section";
import { Steps } from "nextra/components";
import Link from "next/link";

export default async function Home() {
  return (
    <>
      <div className="relative">
        <GridPattern />
        <div className="px-4 pt-16 pb-8 sm:pt-24 lg:px-8">
          <div className="flex w-full flex-col items-center justify-between">
            <div>
              <h1 className="mx-auto max-w-2xl text-center text-6xl leading-none font-extrabold tracking-tighter text-black sm:text-7xl dark:text-white">
                The easiest way to manage your
                <GradientText
                  text="container updates."
                  className="mx-auto w-fit"
                  innerClassName="bg-linear-to-r/oklch from-blue-500 to-green-500"
                  blur={30}
                />
              </h1>
              <h3 className="mx-auto mt-6 max-w-3xl text-center text-xl leading-tight font-medium text-gray-400">
                Cup is a small utility with a big impact. Simplify your
                container management workflow with fast and efficient update
                checking, a full-featured CLI and web interface, and more.
              </h3>
            </div>
            <div className="mt-8 grid w-fit grid-cols-2 gap-4 *:flex *:items-center *:gap-2 *:rounded-lg *:px-3 *:py-2">
              <Link
                href="/docs"
                className="hide-focus group h-full bg-black text-white dark:bg-white dark:text-black"
              >
                Get started
                <IconArrowRight className="ml-auto mr-1 transition-transform duration-300 ease-out group-hover:translate-x-1 group-focus:translate-x-1 dark:!text-black" />
              </Link>
              <a
                href="https://github.com/sergi0g/cup"
                target="_blank"
                className="hide-focus h-full text-nowrap border border-neutral-400 transition-colors duration-200 ease-in-out hover:border-neutral-600 focus:border-neutral-600 dark:border-neutral-600 hover:dark:border-neutral-400 hover:dark:shadow-sm hover:dark:shadow-neutral-600 focus:dark:border-neutral-400"
              >
                Star on GitHub
                <GitHubIcon className="ml-auto size-4 md:size-5" />
              </a>
            </div>
          </div>
        </div>
        <div className="py-10 flex translate-y-32 justify-center" id="hero">
          <Browser />
        </div>
      </div>
      <Section
        title="Powerful at its core."
        className="bg-gradient-to-r from-red-500 to-amber-500"
      >
        <Card
          name="100% Safe Code"
          icon={IconLockCheck}
          description="Built with safe Rust and Typescript to ensure security and reliability."
        />
        <Card
          name="Lightning Fast Performance"
          icon={IconBolt}
          description="Heavily optimized to squeeze out every last drop of performance. Each release is extensively benchmarked and profiled so that you'll never have to stare at a loading spinner for long."
        />
        <Card
          name="Lightweight"
          icon={IconFeather}
          description="No runtimes or libraries are needed. All you need is the 5.1 MB static binary that works out of the box on any system."
        />
      </Section>
      <Section
        title="Efficient, yet flexible."
        className="bg-gradient-to-r from-blue-500 to-indigo-500"
      >
        <Card
          name="JSON output"
          description="Connect Cup to your favorite intergrations with JSON output for the CLI and an API for the server. Now go make that cool dashboard you've been dreaming of!"
          icon={IconBraces}
        />
        <Card
          name="Both CLI and web interface"
          description="Whether you prefer the command line or the web, Cup runs wherever you choose."
          icon={IconDevices}
        />
        <Card
          name="Configurable"
          description="The simple configuration file provides you with all the tools you need to specify a custom Docker socket, manage registry connection options, choose a theme for the web interface and more."
          icon={IconAdjustments}
        />
      </Section>
      <div className="relative py-24 border-t border-t-neutral-300 dark:border-t-neutral-600/30 text-black dark:text-white">
        <GridPattern />
        <div className="mx-auto flex w-full max-w-screen-xl flex-col items-center">
          <p className="mb-8 text-center text-3xl font-bold">
            Still not convinced? Try it out now!
          </p>
          <div>
            <Steps>
              <h3 className="mb-2">Open a terminal and run</h3>
              <CopyableCode>
                docker run --rm -t -v /var/run/docker.sock:/var/run/docker.sock
                -p 8000:8000 ghcr.io/sergi0g/cup serve
              </CopyableCode>
              <h3 className="mb-2">Open the dashboard in your browser</h3>
              <p>
                Visit{" "}
                <a href="http://localhost:8000" className="underline">
                  http://localhost:8000
                </a>{" "}
                in your browser to try it out!
              </p>
            </Steps>
          </div>
        </div>
      </div>
    </>
  );
}
