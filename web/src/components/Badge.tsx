import { theme } from "../theme";
import { ReactNode } from "react";
import { cn } from "../utils";

export default function Badge({ children, className }: { children: ReactNode, className?: string }) {
  return (
    <span
      className={cn(`inline-flex items-center rounded-full bg-${theme}-50 px-2 py-1 text-xs font-medium text-${theme}-700 ring-1 ring-inset ring-${theme}-700/10 dark:bg-${theme}-400/10 dark:text-${theme}-400 dark:ring-${theme}-400/30 break-keep`, className)}
    >
      {children}
    </span>
  );
}
