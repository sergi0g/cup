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
      <div className="xl:px-8 px-6 py-4 gap-y-2 gap-x-4 justify-between align-baseline flex flex-col h-full">
        <dt
          className={`text-${theme}-500 dark:text-${theme}-400 leading-6 font-medium`}
        >
          {name}
        </dt>
        <div className="flex gap-1 justify-between items-center">
          <dd className="text-black dark:text-white tracking-tight leading-10 font-medium text-3xl w-full">
            {value}
          </dd>
          {name == "Monitored images" && (
            <IconEyeFilled className="size-6 text-black dark:text-white shrink-0" />
          )}
          {name == "Up to date" && (
            <IconCircleCheckFilled className="size-6 text-green-500 shrink-0" />
          )}
          {name == "Update available" && (
            <IconCircleArrowUpFilled className="size-6 text-blue-500 shrink-0" />
          )}
          {name == "Unknown" && (
            <IconHelpCircleFilled className="size-6 text-gray-500 shrink-0" />
          )}
        </div>
      </div>
    </div>
  );
}
