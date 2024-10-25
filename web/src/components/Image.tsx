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
        className={`*:py-4 *:px-6 *:flex *:items-center *:gap-3 w-full`}
      >
        <li className="break-all">
          <IconCube className="size-6 shrink-0" />
          {data.reference}
          {data.result.has_update == false && (
            <WithTooltip
              text="Up to date"
              className="text-green-500 ml-auto size-6 shrink-0"
            >
              <IconCircleCheckFilled />
            </WithTooltip>
          )}
          {data.result.has_update == true && (
            <WithTooltip
              text="Update available"
              className="text-blue-500 ml-auto size-6 shrink-0"
            >
              <IconCircleArrowUpFilled />
            </WithTooltip>
          )}
          {data.result.has_update == null && (
            <WithTooltip
              text="Unknown"
              className="text-gray-500 ml-auto size-6 shrink-0"
            >
              <IconHelpCircleFilled />
            </WithTooltip>
          )}
        </li>
      </button>
      <Dialog open={open} onClose={setOpen} className="relative z-10">
        <DialogBackdrop
          transition
          className={`fixed inset-0 bg-${theme}-500 dark:bg-${theme}-950 !bg-opacity-75 transition-opacity data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in`}
        />
        <div className="fixed inset-0 z-10 w-screen overflow-y-auto">
          <div className="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
            <DialogPanel
              transition
              className={`relative transform overflow-hidden rounded-lg bg-white dark:bg-${theme}-900 dark:text-white text-left shadow-xl transition-all data-[closed]:translate-y-4 data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in sm:my-8 sm:w-full sm:max-w-lg md:max-w-xl lg:max-w-2xl data-[closed]:sm:translate-y-0 data-[closed]:sm:scale-95`}
            >
              <div
                className={`py-4 px-6 flex flex-col gap-3 text-${theme}-400 dark:text-${theme}-600`}
              >
                <div className="flex items-center gap-3 mb-4">
                  <IconCube className="size-6 shrink-0 text-black dark:text-white" />
                  <DialogTitle className="text-black dark:text-white">
                    {url ? (
                      <>
                        <a
                          href={url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="inline-block after:bg-white after:h-[2px] after:bottom-[4px] after:left-0 after:scale-x-0 after:block after:relative after:w-full after:transition-transform after:duration-300 hover:after:scale-x-100"
                        >
                          {data.reference}
                        </a>
                        <IconArrowUpRight className="inline-block size-3" />
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
                  {data.result.has_update == false && (
                    <>
                      <IconCircleCheckFilled className="text-green-500 size-6 shrink-0" />
                      Up to date
                    </>
                  )}
                  {data.result.has_update == true && (
                    <>
                      <IconCircleArrowUpFilled className="text-blue-500 size-6 shrink-0" />
                      Update available
                    </>
                  )}
                  {data.result.has_update == null && (
                    <>
                      <IconHelpCircleFilled className="text-gray-500 size-6 shrink-0" />
                      Unknown
                    </>
                  )}
                </div>
                <div className="flex items-center gap-3 mb-4">
                  <IconStopwatch className="text-gray-500 size-6 shrink-0" />
                  Checked in {data.time} ms
                </div>
                {data.result.error && (
                  <div className="bg-yellow-400/10 flex items-center gap-3 overflow-hidden break-all rounded-md px-3 py-2 mb-4">
                    <IconAlertTriangleFilled className="text-yellow-500 size-5 shrink-0" />
                    {data.result.error}
                  </div>
                )}
                {data.result.has_update && (
                  <div className="flex flex-col gap-1">
                    Pull command
                    <div
                      className={`bg-${theme}-50 dark:bg-${theme}-950 text-gray-500 flex items-center rounded-md px-3 py-2 font-mono mb-4 group relative`}
                    >
                      <p className="overflow-scroll">
                        docker pull {data.reference}
                      </p>
                      {navigator.clipboard &&
                        (copySuccess ? (
                          <IconCheck className="absolute right-3" />
                        ) : (
                          <button
                            className="absolute right-3 opacity-0 group-hover:opacity-100 transition-opacity duration-50"
                            onClick={handleCopy(
                              `docker pull ${data.reference}`
                            )}
                          >
                            <IconCopy />
                          </button>
                        ))}
                    </div>
                  </div>
                )}
                <div className="flex flex-col gap-1">
                  Local digests
                  <div
                    className={`bg-${theme}-50 dark:bg-${theme}-950 text-gray-500 rounded-md px-3 py-2 font-mono scrollable`}
                  >
                    <p className="overflow-x-scroll">
                      {data.local_digests.join("\n")}
                    </p>
                  </div>
                </div>
                {data.remote_digest && (
                  <div className="flex flex-col gap-1">
                    Remote digest
                    <div
                      className={`bg-${theme}-50 dark:bg-${theme}-950 text-gray-500 rounded-md px-3 py-2 font-mono`}
                    >
                      <p className="overflow-x-scroll">{data.remote_digest}</p>
                    </div>
                  </div>
                )}
              </div>
            </DialogPanel>
          </div>
        </div>
      </Dialog>
    </>
  );
}
