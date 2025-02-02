"use client";

import { IconCopy, IconCopyCheck } from "@tabler/icons-react";
import { useState } from "react";

export default function CopyableCode({ children }: { children: string }) {
  const [success, setSuccess] = useState(false);
  const handleClick = () => {
    navigator.clipboard.writeText(children);
    setSuccess(true);
    setTimeout(() => setSuccess(false), 3000);
  };
  return (
    <div className="relative rounded-md xl:w-auto">
      <button
        className="hover:bg-black/10 dark:hover:bg-black/60 flex w-full items-center justify-center gap-4 rounded-md border border-black/10 bg-black/5 px-8 py-3 font-mono text-sm font-medium text-black/70 transition-colors duration-200 md:px-10 md:py-3 md:text-base md:leading-6 dark:border-white/15 dark:bg-black dark:text-gray-300 backdrop-blur-md"
        onClick={handleClick}
      >
        {children}
        {success ? (
          <IconCopyCheck className="stroke-black/40 dark:stroke-white/50" />
        ) : (
          <IconCopy className="stroke-black/40 dark:stroke-white/50" />
        )}
      </button>
    </div>
  );
}
