import { useState } from "react";
import {
  Dialog,
  DialogBackdrop,
  DialogPanel,
  DialogTitle,
} from "@headlessui/react";
import { WithTooltip } from "./Tooltip";
import type { Image } from "../types";
import { theme } from "../theme";
import { CodeBlock } from "./CodeBlock";
import { Box, CircleArrowUp, CircleCheck, HelpCircle, Timer, TriangleAlert, X } from "lucide-react";

const clickable_registries = [
  "registry-1.docker.io",
  "ghcr.io",
  "quay.io",
  "gcr.io",
]; // Not all registries redirect to an info page when visiting the image reference in a browser (e.g. Gitea and derivatives), so we only enable clicking those who do.

export default function Image({ data }: { data: Image }) {
  const [open, setOpen] = useState(false);
  const handleOpen = () => {
    setOpen(true);
  };
  const handleClose = () => {
    setOpen(false);
  };
  const new_reference =
    data.result.info?.type == "version"
      ? data.reference.split(":")[0] + ":" + data.result.info.new_tag
      : data.reference;
  let url: string | null = null;
  if (clickable_registries.includes(data.parts.registry)) {
    switch (data.parts.registry) {
      case "registry-1.docker.io":
        url = `https://hub.docker.com/r/${data.parts.repository}`;
        break;
      default:
        url = `https://${data.parts.registry}/${data.parts.repository}`;
        break;
    }
  }
  return (
    <>
      <button onClick={handleOpen} className="w-full">
        <li className="flex items-center gap-4 break-all px-6 py-4 text-start">
          <Box className="size-6 shrink-0" />
          {data.reference}
          <Icon data={data} />
        </li>
      </button>
      <Dialog open={open} onClose={setOpen} className="relative z-10">
        <DialogBackdrop
          transition
          className={`fixed inset-0 bg-${theme}-500 dark:bg-${theme}-950 !bg-opacity-75 transition-opacity data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in`}
        />
        <div className="fixed inset-0 z-10 w-screen overflow-y-auto">
          <div className="flex min-h-full items-end justify-center text-center sm:items-center sm:p-0">
            <DialogPanel
              transition
              className={`relative transform overflow-hidden rounded-t-lg bg-white md:rounded-lg dark:bg-${theme}-900 w-full text-left shadow-xl transition-all data-[closed]:translate-y-4 data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in sm:my-8 sm:w-full sm:max-w-lg data-[closed]:sm:translate-y-0 data-[closed]:sm:scale-95 md:max-w-xl lg:max-w-2xl dark:text-white`}
            >
              <div
                className={`flex flex-col gap-3 px-6 py-4 text-${theme}-400 dark:text-${theme}-600`}
              >
                <div className="mb-4 flex items-center gap-3">
                  <Box className="size-6 shrink-0 text-black dark:text-white" />
                  <DialogTitle className="text-black dark:text-white">
                    {url ? (
                      <>
                        <a
                          href={url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className={`group flex w-fit items-center justify-center gap-1 text-black hover:underline dark:text-white`}
                        >
                          <span>{data.reference}</span>
                          <svg
                            viewBox="0 0 12 12"
                            fill="none"
                            width="10px"
                            xmlns="http://www.w3.org/2000/svg"
                            className="transition-all duration-100 group-hover:rotate-45"
                          >
                            <path
                              d="M11 9.283V1H2.727v1.44h5.83L1 9.99 2.01 11l7.556-7.55v5.833H11Z"
                              fill="currentColor"
                            ></path>
                          </svg>
                        </a>
                      </>
                    ) : (
                      data.reference
                    )}
                  </DialogTitle>
                  <button onClick={handleClose} className="ml-auto">
                    <X className="size-6 shrink-0 text-gray-500" />
                  </button>
                </div>
                <div className="flex items-center gap-3">
                  <DialogIcon data={data} />
                </div>
                <div className="mb-4 flex items-center gap-3">
                  <Timer className="size-6 shrink-0 text-gray-500" />
                  <span>
                    Checked in <b>{data.time}</b> ms
                  </span>
                </div>
                {data.result.error && (
                  <div className="break-before mb-4 flex items-center gap-3 overflow-hidden rounded-md bg-yellow-400/10 px-3 py-2">
                    <TriangleAlert className="size-6 shrink-0 text-yellow-500" />
                    {data.result.error}
                  </div>
                )}
                {data.result.has_update && (
                  <div className="flex flex-col gap-1">
                    Pull command
                    <CodeBlock enableCopy>{new_reference}</CodeBlock>
                  </div>
                )}
                <div className="flex flex-col gap-1">
                  {data.result.info?.type == "digest" && (
                    <>
                      {data.result.info.local_digests.length > 1
                        ? "Local digests"
                        : "Local digest"}
                      <CodeBlock enableCopy>
                        {data.result.info.local_digests.join("\n")}
                      </CodeBlock>
                      {data.result.info.remote_digest && (
                        <div className="flex flex-col gap-1">
                          Remote digest
                          <CodeBlock enableCopy>
                            {data.result.info.remote_digest}
                          </CodeBlock>
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>
            </DialogPanel>
          </div>
        </div>
      </Dialog>
    </>
  );
}

function Icon({ data }: { data: Image }) {
  switch (data.result.has_update) {
    case null:
      return (
        <WithTooltip
          text="Unknown"
          className="ml-auto size-6 shrink-0 text-gray-500"
        >
          <HelpCircle />
        </WithTooltip>
      );
    case false:
      return (
        <WithTooltip
          text="Up to date"
          className="ml-auto size-6 shrink-0 text-green-500"
        >
          <CircleCheck />
        </WithTooltip>
      );
    case true:
      if (data.result.info?.type === "version") {
        switch (data.result.info.version_update_type) {
          case "major":
            return (
              <WithTooltip
                text="Major Update"
                className="ml-auto size-6 shrink-0 text-red-500"
              >
                <CircleArrowUp />
              </WithTooltip>
            );
          case "minor":
            return (
              <WithTooltip
                text="Minor Update"
                className="ml-auto size-6 shrink-0 text-yellow-500"
              >
                <CircleArrowUp />
              </WithTooltip>
            );
          case "patch":
            return (
              <WithTooltip
                text="Patch Update"
                className="ml-auto size-6 shrink-0 text-blue-500"
              >
                <CircleArrowUp />
              </WithTooltip>
            );
        }
      } else if (data.result.info?.type === "digest") {
        return (
          <WithTooltip
            text="Update available"
            className="ml-auto size-6 shrink-0 text-blue-500"
          >
            <CircleArrowUp />
          </WithTooltip>
        );
      }
  }
}

function DialogIcon({ data }: { data: Image }) {
  switch (data.result.has_update) {
    case null:
      return (
        <>
          <HelpCircle className="size-6 shrink-0 text-gray-500" />
          Unknown
        </>
      );
    case false:
      return (
        <>
          <CircleCheck className="size-6 shrink-0 text-green-500" />
          Up to date
        </>
      );
    case true:
      if (data.result.info?.type === "version") {
        switch (data.result.info.version_update_type) {
          case "major":
            return (
              <>
                <CircleArrowUp className="size-6 shrink-0 text-red-500" />
                Major update
              </>
            );
          case "minor":
            return (
              <>
                <CircleArrowUp className="size-6 shrink-0 text-yellow-500" />
                Minor update
              </>
            );
          case "patch":
            return (
              <>
                <CircleArrowUp className="size-6 shrink-0 text-blue-500" />
                Patch update
              </>
            );
        }
      } else if (data.result.info?.type === "digest") {
        return (
          <>
            <CircleArrowUp className="size-6 shrink-0 text-blue-500" />
            Update available
          </>
        );
      }
  }
}
