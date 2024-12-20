import { useState } from "react";
import { theme } from "../theme";
import { IconCheck, IconClipboard } from "@tabler/icons-react";

export function PullCommand({ reference }: { reference: string }) {
  const [copySuccess, setCopySuccess] = useState(false);
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
    <div className="flex flex-col gap-1">
      Pull command
      <div
        className={`w-full group relative flex items-center rounded-lg bg-${theme}-100 px-3 py-2 font-mono text-${theme}-700 dark:bg-${theme}-950 dark:text-${theme}-300`}
      >
        <p className="overflow-scroll">docker pull {reference}</p>
        {navigator.clipboard &&
          (copySuccess ? (
            <IconCheck className="absolute right-3 size-7 bg-neutral-100 pl-2 dark:bg-neutral-950" />
          ) : (
            <button
              className="duration-50 absolute right-3 opacity-0 transition-opacity group-hover:opacity-100"
              onClick={handleCopy(`docker pull ${reference}`)}
            >
              <IconClipboard className="size-5"/>
            </button>
          ))}
      </div>
    </div>
  );
}
