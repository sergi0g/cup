import { useState } from "react";
import {
  Dialog,
  DialogBackdrop,
  DialogPanel,
  DialogTitle,
} from "@headlessui/react";
import {
  IconAlertTriangleFilled,
  IconArrowUpRight,
  IconCheck,
  IconCircleArrowUpFilled,
  IconCircleCheckFilled,
  IconCopy,
  IconCube,
  IconHelpCircleFilled,
  IconStopwatch,
  IconX,
} from "@tabler/icons-react";
import { WithTooltip } from "./Tooltip";
import type { Image } from "../types";
import { theme } from "../theme";

const clickable_registries = [
  "registry-1.docker.io",
  "ghcr.io",
  "quay.io",
  "gcr.io",
]; // Not all registries redirect to an info page when visiting the image reference in a browser (e.g. Gitea and derivatives), so we only enable clicking those who do.

export default function Image({ data }: { data: Image }) {
  const [open, setOpen] = useState(false);
  const [copySuccess, setCopySuccess] = useState(false);
  const handleOpen = () => {
    setOpen(true);
  };
  const handleClose = () => {
    setOpen(false);
  };
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
  var url: string | null = null;
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
      <button
        onClick={handleOpen}
        className={`w-full *:flex *:items-center *:gap-3 *:px-6 *:py-4`}
      >
        <li className="break-all text-start">
          <IconCube className="size-6 shrink-0" />
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
              className={`relative transform overflow-hidden rounded-t-lg bg-white md:rounded-lg dark:bg-${theme}-900 text-left shadow-xl transition-all data-[closed]:translate-y-4 data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in sm:my-8 sm:w-full sm:max-w-lg data-[closed]:sm:translate-y-0 data-[closed]:sm:scale-95 md:max-w-xl lg:max-w-2xl dark:text-white`}
            >
              <div
                className={`flex flex-col gap-3 px-6 py-4 text-${theme}-400 dark:text-${theme}-600`}
              >
                <div className="mb-4 flex items-center gap-3">
                  <IconCube className="size-6 shrink-0 text-black dark:text-white" />
                  <DialogTitle className="text-black dark:text-white">
                    {url ? (
                      <>
                        <a
                          href={url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="after:relative after:bottom-[1px] after:left-0 after:block after:h-[2px] after:w-full after:scale-x-0 after:bg-black after:transition-transform after:duration-300 hover:after:scale-x-100 after:dark:bg-white"
                        >
                          {data.reference}
                          <span className="inline-flex align-text-top">
                            <IconArrowUpRight className="size-3 shrink-0" />
                          </span>
                        </a>
                      </>
                    ) : (
                      data.reference
                    )}
                  </DialogTitle>
                  <button onClick={handleClose} className="ml-auto">
                    <IconX className="size-6 shrink-0 text-gray-500" />
                  </button>
                </div>
                <div className="flex items-center gap-3">
                  <DialogIcon data={data} />
                </div>
                <div className="mb-4 flex items-center gap-3">
                  <IconStopwatch className="size-6 shrink-0 text-gray-500" />
                  <span>
                    Checked in <b>{data.time}</b> ms
                  </span>
                </div>
                {data.result.error && (
                  <div className="mb-4 flex items-center gap-3 overflow-hidden break-all rounded-md bg-yellow-400/10 px-3 py-2">
                    <IconAlertTriangleFilled className="size-5 shrink-0 text-yellow-500" />
                    {data.result.error}
                  </div>
                )}
                {data.result.has_update && (
                  <div className="flex flex-col gap-1">
                    Pull command
                    <div
                      className={`bg-${theme}-100 dark:bg-${theme}-950 group relative mb-4 flex items-center rounded-md px-3 py-2 font-mono text-gray-500`}
                    >
                      <p className="overflow-scroll">
                      docker pull {data.result.info?.type == "version" ? data.reference.replace(data.parts.tag, data.result.info.new_version) : data.reference}
                      </p>
                      {navigator.clipboard &&
                        (copySuccess ? (
                          <IconCheck className="absolute right-3" />
                        ) : (
                          <button
                            className="duration-50 absolute right-3 opacity-0 transition-opacity group-hover:opacity-100"
                            onClick={handleCopy(
                              `docker pull ${data.result.info?.type == "version" ? data.reference.replace(data.parts.tag, data.result.info.new_version) : data.reference}`,
                            )}
                          >
                            <IconCopy />
                          </button>
                        ))}
                    </div>
                  </div>
                )}
                <div className="flex flex-col gap-1">
                  {data.result.info?.type == "digest" && (
                    <>
                      {data.result.info.local_digests.length > 1
                        ? "Local digests"
                        : "Local digest"}
                      <div
                        className={`bg-${theme}-100 dark:bg-${theme}-950 scrollable rounded-md px-3 py-2 font-mono text-gray-500`}
                      >
                        <p className="overflow-x-scroll">
                          {data.result.info.local_digests.join("\n")}
                        </p>
                      </div>
                      {data.result.info.remote_digest && (
                        <div className="flex flex-col gap-1">
                          Remote digest
                          <div
                            className={`bg-${theme}-100 dark:bg-${theme}-950 rounded-md px-3 py-2 font-mono text-gray-500`}
                          >
                            <p className="overflow-x-scroll">
                              {data.result.info.remote_digest}
                            </p>
                          </div>
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
          <IconHelpCircleFilled />
        </WithTooltip>
      );
    case false:
      return (
        <WithTooltip
          text="Up to date"
          className="ml-auto size-6 shrink-0 text-green-500"
        >
          <IconCircleCheckFilled />
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
                <IconCircleArrowUpFilled />
              </WithTooltip>
            );
          case "minor":
            return (
              <WithTooltip
                text="Minor Update"
                className="ml-auto size-6 shrink-0 text-yellow-500"
              >
                <IconCircleArrowUpFilled />
              </WithTooltip>
            );
          case "patch":
            return (
              <WithTooltip
                text="Patch Update"
                className="ml-auto size-6 shrink-0 text-blue-500"
              >
                <IconCircleArrowUpFilled />
              </WithTooltip>
            );
        }
      } else if (data.result.info?.type === "digest") {
        return (
          <WithTooltip
            text="Update available"
            className="ml-auto size-6 shrink-0 text-blue-500"
          >
            <IconCircleArrowUpFilled />
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
          <IconHelpCircleFilled className="size-6 shrink-0 text-gray-500" />
          Unknown
        </>
      );
    case false:
      return (
        <>
          <IconCircleCheckFilled className="size-6 shrink-0 text-green-500" />
          Up to date
        </>
      );
    case true:
      if (data.result.info?.type === "version") {
        switch (data.result.info.version_update_type) {
          case "major":
            return (
              <>
                <IconCircleArrowUpFilled className="size-6 shrink-0 text-red-500" />
                Major update
              </>
            );
          case "minor":
            return (
              <>
                <IconCircleArrowUpFilled className="size-6 shrink-0 text-yellow-500" />
                Minor update
              </>
            );
          case "patch":
            return (
              <>
                <IconCircleArrowUpFilled className="size-6 shrink-0 text-blue-500" />
                Patch update
              </>
            );
        }
      } else if (data.result.info?.type === "digest") {
        return (
          <>
            <IconCircleArrowUpFilled className="size-6 shrink-0 text-blue-500" />
            Update available
          </>
        );
      }
  }
}
