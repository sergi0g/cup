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
        <li className="break-all">
          <IconCube className="size-6 shrink-0" />
          {data.reference}
          {data.result.has_update == false && (
            <WithTooltip
              text="Up to date"
              className="ml-auto size-6 shrink-0 text-green-500"
            >
              <IconCircleCheckFilled />
            </WithTooltip>
          )}
          {data.result.has_update == true && (
            <WithTooltip
              text="Update available"
              className="ml-auto size-6 shrink-0 text-blue-500"
            >
              <IconCircleArrowUpFilled />
            </WithTooltip>
          )}
          {data.result.has_update == null && (
            <WithTooltip
              text="Unknown"
              className="ml-auto size-6 shrink-0 text-gray-500"
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
              className={`relative transform overflow-hidden rounded-lg bg-white dark:bg-${theme}-900 text-left shadow-xl transition-all data-[closed]:translate-y-4 data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in sm:my-8 sm:w-full sm:max-w-lg data-[closed]:sm:translate-y-0 data-[closed]:sm:scale-95 md:max-w-xl lg:max-w-2xl dark:text-white`}
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
                          className="inline-block after:relative after:bottom-[1px] after:left-0 after:block after:h-[2px] after:w-full after:scale-x-0 after:bg-black after:dark:bg-white after:transition-transform after:duration-300 hover:after:scale-x-100"
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
                      <IconCircleCheckFilled className="size-6 shrink-0 text-green-500" />
                      Up to date
                    </>
                  )}
                  {data.result.has_update == true && (
                    <>
                      <IconCircleArrowUpFilled className="size-6 shrink-0 text-blue-500" />
                      Update available
                    </>
                  )}
                  {data.result.has_update == null && (
                    <>
                      <IconHelpCircleFilled className="size-6 shrink-0 text-gray-500" />
                      Unknown
                    </>
                  )}
                </div>
                <div className="mb-4 flex items-center gap-3">
                  <IconStopwatch className="size-6 shrink-0 text-gray-500" />
                  Checked in {data.time} ms
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
                        docker pull {data.reference}
                      </p>
                      {navigator.clipboard &&
                        (copySuccess ? (
                          <IconCheck className="absolute right-3" />
                        ) : (
                          <button
                            className="duration-50 absolute right-3 opacity-0 transition-opacity group-hover:opacity-100"
                            onClick={handleCopy(
                              `docker pull ${data.reference}`,
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
                    className={`bg-${theme}-100 dark:bg-${theme}-950 scrollable rounded-md px-3 py-2 font-mono text-gray-500`}
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
                      className={`bg-${theme}-100 dark:bg-${theme}-950 rounded-md px-3 py-2 font-mono text-gray-500`}
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
