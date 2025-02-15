import { ReactNode, useState } from "react";
import { theme } from "../theme";
import { Clipboard, ClipboardCheck } from "lucide-react";

export function CodeBlock({
  children,
  enableCopy,
}: {
  children: ReactNode;
  enableCopy?: boolean;
}) {
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
    <div
      className={`group relative flex w-full items-center rounded-lg bg-${theme}-100 px-3 py-2 font-mono text-${theme}-700 dark:bg-${theme}-950 dark:text-${theme}-300`}
    >
      <p className="overflow-scroll">{children}</p>
      {enableCopy &&
        navigator.clipboard &&
        (copySuccess ? (
          <ClipboardCheck
            className={`absolute right-3 size-7 bg-${theme}-100 py-1 pl-2 dark:bg-${theme}-950`}
          />
        ) : (
          <button
            className={`duration-50 absolute right-3 bg-${theme}-100 py-1 pl-2 opacity-0 transition-opacity group-hover:opacity-100 dark:bg-${theme}-950`}
            onClick={handleCopy(`docker pull ${children}`)}
          >
            <Clipboard className="size-5" />
          </button>
        ))}
    </div>
  );
}
