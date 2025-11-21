import { ArrowRight } from "lucide-react";
import { theme } from "../theme";

export default function Badge({ from, to }: { from: string; to: string }) {
  return (
    <span
      className={`hidden sm:inline-flex items-center rounded-full bg-${theme}-50 px-2 py-1 text-xs font-medium text-${theme}-700 ring-1 ring-inset ring-${theme}-700/10 dark:bg-${theme}-400/10 dark:text-${theme}-400 dark:ring-${theme}-400/30 break-keep`}
    >
      {from}
      <ArrowRight className="size-3" />
      {to}
    </span>
  );
}
