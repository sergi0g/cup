import React from "react";
import "./styles.css";

import { Browser } from "../Browser";
import { Card } from "../Card";
import {
  IconAdjustments,
  IconArrowRight,
  IconBarrierBlockOff,
  IconBolt,
  IconFeather,
  IconGitMerge,
  IconPuzzle,
  IconServer,
  IconTerminal,
} from "@tabler/icons-react";
import { GitHubIcon } from "nextra/icons";
import { GridPattern } from "../GridPattern";
import { GradientText } from "../GradientText";
import Link from "next/link";

export default async function Home() {
  return (
    <>
      <div className="relative home bg-radial-[ellipse_at_center] from-transparent from-20% to-white dark:to-black">
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
              <h3 className="mx-auto mt-6 max-w-3xl text-center text-xl leading-tight font-medium text-neutral-500 dark:text-neutral-400">
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
                className="hide-focus h-full bg-white dark:bg-black text-nowrap border border-black/15 transition-colors duration-200 ease-in-out hover:border-black/40 dark:border-white/15 hover:dark:border-white/40 hover:dark:shadow-sm focus:dark:border-white/30"
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
      <div className="bg-white dark:bg-black py-12 px-8 w-full">
        <div className="flex h-full w-full items-center justify-center">
          <div className="grid md:grid-cols-2 md:grid-rows-4 lg:grid-cols-4 lg:grid-rows-2 w-full max-w-7xl gap-px border border-transparent bg-black/10 dark:bg-white/10">
            <Card
              name="Built for speed."
              icon={IconBolt}
              description="Cup is written in Rust and every release goes through extensive profiling to squeeze out every last drop of performance."
            />
            <Card
              name="Configurable."
              icon={IconAdjustments}
              description="Make Cup yours with the extensive configuration options available. Customize and tailor it to your needs."
            />
            <Card
              name="Extend it."
              icon={IconPuzzle}
              description="JSON output enables you to connect Cup with your favorite integrations, build automations and more."
            />
            <Card
              name="CLI available."
              icon={IconTerminal}
              description="Do you like terminals? Cup has a CLI. Check for updates quickly without spinning up a server."
            />
            <Card
              name="Multiple servers."
              icon={IconServer}
              description="Run multiple Cup instances and effortlessly check on them through one web interface."
            />
            <Card
              name="Unstoppable."
              icon={IconBarrierBlockOff}
              description="Cup is designed to check for updates without using up any rate limits. 10 images per hour won't be a problem, even with 100 images."
            />
            <Card
              name="Lightweight."
              icon={IconFeather}
              description="No need for a powerful server and endless storage. The tiny 5.4 MB binary won't hog your CPU and memory."
            />
            <Card
              name="Open source."
              icon={IconGitMerge}
              description="All source code is publicly available in our GitHub repository. We're looking for contributors!"
            />
          </div>
        </div>
      </div>
    </>
  );
}
