import {
  IconCircleArrowUpFilled,
  IconCircleCheckFilled,
  IconEyeFilled,
  IconHelpCircleFilled,
} from "@tabler/icons-react";
import { theme } from "../theme";

export default function Statistic({
  name,
  value,
}: {
  name: string;
  value: number;
}) {
  name = name.replaceAll("_", " ");
  name = name.slice(0, 1).toUpperCase() + name.slice(1); // Capitalize name
  return (
    <div
      className={`before:bg-${theme}-200 before:dark:bg-${theme}-800 after:bg-${theme}-200 after:dark:bg-${theme}-800 gi`}
    >
      <div className="flex h-full flex-col justify-between gap-x-4 gap-y-2 px-6 py-4 align-baseline xl:px-8">
        <dt
          className={`text-${theme}-500 dark:text-${theme}-400 font-medium leading-6`}
        >
          {name}
        </dt>
        <div className="flex items-center justify-between gap-1">
          <dd className="w-full text-3xl font-medium leading-10 tracking-tight text-black dark:text-white">
            {value}
          </dd>
          {name == "Monitored images" && (
            <IconEyeFilled className="size-6 shrink-0 text-black dark:text-white" />
          )}
          {name == "Up to date" && (
            <IconCircleCheckFilled className="size-6 shrink-0 text-green-500" />
          )}
          {name == "Update available" && (
            <IconCircleArrowUpFilled className="size-6 shrink-0 text-blue-500" />
          )}
          {name == "Unknown" && (
            <IconHelpCircleFilled className="size-6 shrink-0 text-gray-500" />
          )}
        </div>
      </div>
    </div>
  );
}
