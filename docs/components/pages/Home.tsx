import React, { useState, useEffect } from "react";
import {
  IconAdjustments,
  IconArrowRight,
  IconBolt,
  IconBraces,
  IconCheck,
  IconClipboard,
  IconDevices,
  IconFeather,
  IconLockCheck,
} from "@tabler/icons-react";
import { GitHubIcon } from "../../assets/GitHubIcon";
import { GridPattern } from "../GridPattern";
import { Section } from "../Section";
import { GradientText } from "../GradientText";
import Image from "next/image";
import screenshot_light from "../../assets/screenshot_light.png";
import screenshot_dark from "../../assets/screenshot_dark.png";
import { Step } from "../Step";
import { Card } from "../Card";

export function Home() {
  const [copySuccess, setCopySuccess] = useState(false);
  const [isBrowser, setIsBrowser] = useState(false); // To prevent hydration mismatch
  useEffect(() => setIsBrowser(true));
  const handleCopy = (text: string) => {
    return () => {
      navigator.clipboard.writeText(text).then(() => {
        setCopySuccess(true);
        setTimeout(() => {
          setCopySuccess(false);
        }, 3000);
      });
    };
  };
  return (
    <div className="home-animation" style={{ opacity: 0 }}>
      <div className="relative h-full overflow-x-hidden p-4 pt-10 lg:pt-20">
        <GridPattern />
        <div className="mx-auto h-full min-h-svh w-full max-w-screen-xl md:h-[46rem] md:min-h-0">
          <div className="grid gap-8 lg:grid-cols-2">
            <div className="flex max-w-3xl flex-col gap-8">
              <div className="text-6xl font-extrabold leading-none tracking-tighter sm:text-7xl">
                The easiest way to manage your
                <GradientText
                  text="container updates."
                  innerClassName="bg-gradient-to-r from-blue-500 to-green-500"
                  blur={30}
                />
              </div>
              <h3 className="text-xl text-neutral-600 dark:text-neutral-400">
                Cup is a small utility with a big impact. Simplify your
                container management workflow with fast and efficient update
                checking, a full-featured CLI and web inteface, and more.
              </h3>
              <div className="*:-0 mt-auto grid w-fit grid-cols-2 gap-4 *:flex *:items-center *:gap-2 *:rounded-lg *:px-3 *:py-2">
                <a
                  href="/docs"
                  className="hide-focus group h-full bg-black text-white dark:bg-white dark:text-black"
                >
                  Get started
                  <IconArrowRight className="ml-auto mr-1 transition-transform duration-300 ease-out group-hover:translate-x-1 group-focus:translate-x-1 dark:!text-black" />
                </a>
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
            <div className="h-full">
              <div className="max-h-[33.75rem] max-w-full overflow-hidden rounded-xl border border-neutral-200 dark:border-neutral-800">
                <Image
                  src={screenshot_light}
                  alt="Screenshot of Cup's web interface"
                  className="dark:hidden"
                />
                <Image
                  src={screenshot_dark}
                  alt="Screenshot of Cup's web interface"
                  className="hidden dark:block"
                />
              </div>
            </div>
          </div>
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
          description="Whether you prefer the command line, or the web, Cup runs wherever you choose."
          icon={IconDevices}
        />
        <Card
          name="Configurable"
          description="The simple configuration file provides you with all the tools you need to specify a custom Docker socket, manage registry connection options, choose a theme for the web interface and more."
          icon={IconAdjustments}
        />
      </Section>
      <div className="relative py-24">
        <GridPattern />
        <div className="mx-auto flex w-full max-w-screen-xl flex-col items-center">
          <p className="mb-8 text-center text-3xl font-bold">
            Still not convinced? Try it out now!
          </p>
          <div>
            <Step title="Open a terminal and run" number={1}>
              <div className="group relative mx-auto flex max-w-screen-xl items-center rounded-lg bg-neutral-100 px-3 py-2 font-mono text-neutral-700 dark:bg-neutral-950 dark:text-neutral-300">
                <p className="overflow-scroll">
                docker run --rm -t -v /var/run/docker.sock:/var/run/docker.sock -p 8000:8000 ghcr.io/sergi0g/cup serve
                </p>
                {isBrowser &&
                  navigator.clipboard &&
                  (copySuccess ? (
                    <IconCheck className="absolute right-3 size-7 bg-neutral-100 pl-2 dark:bg-neutral-950" />
                  ) : (
                    <button
                      className="duration-50 absolute right-3 bg-neutral-100 pl-2 opacity-0 transition-opacity group-hover:opacity-100 dark:bg-neutral-950"
                      onClick={handleCopy(
                        "docker run --rm -t -v /var/run/docker.sock:/var/run/docker.sock -p 8000:8000 ghcr.io/sergi0g/cup serve",
                      )}
                    >
                      <IconClipboard className="size-5" />
                    </button>
                  ))}
              </div>
            </Step>
            <Step number={2} title="Open the dashboard in your browser">
              <p>
                Visit{" "}
                <a href="http://localhost:8000" className="underline">
                  http://localhost:8000
                </a>{" "}
                in your browser to try it out!
              </p>
            </Step>
          </div>
        </div>
      </div>
    </div>
  );
}
