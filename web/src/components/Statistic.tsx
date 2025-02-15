import { CircleArrowUp, CircleCheck, Eye, HelpCircle } from "lucide-react";
import { theme } from "../theme";
import { Data } from "../types";

const metricsToShow = [
  "monitored_images",
  "up_to_date",
  "updates_available",
  "unknown",
];

export default function Statistic({
  name,
  metrics,
}: {
  name: keyof Data["metrics"];
  metrics: Data["metrics"];
}) {
  if (!metricsToShow.includes(name)) return null;
  let displayName = name.replaceAll("_", " ");
  displayName = displayName.slice(0, 1).toUpperCase() + displayName.slice(1); // Capitalize name
  return (
    <div
      className={`before:bg-${theme}-200 before:dark:bg-${theme}-800 after:bg-${theme}-200 after:dark:bg-${theme}-800 gi`}
    >
      <div className="flex h-full flex-col justify-between gap-x-4 gap-y-2 px-6 py-4 align-baseline lg:min-h-32">
        <dt
          className={`text-${theme}-500 dark:text-${theme}-400 font-medium leading-6`}
        >
          {displayName}
        </dt>
        <div className="flex items-center justify-between gap-1">
          <dd className="w-full text-3xl font-medium leading-10 tracking-tight text-black dark:text-white">
            {metrics[name]}
          </dd>
          {name === "monitored_images" && (
            <Eye className="size-6 shrink-0 text-black dark:text-white" />
          )}
          {name === "up_to_date" && (
            <CircleCheck className="size-6 shrink-0 text-green-500" />
          )}
          {name === "updates_available" && getUpdatesAvailableIcon(metrics)}
          {name === "unknown" && (
            <HelpCircle className="size-6 shrink-0 text-gray-500" />
          )}
        </div>
      </div>
    </div>
  );
}

function getUpdatesAvailableIcon(metrics: Data["metrics"]) {
  const filteredMetrics = Object.entries(metrics).filter(
    ([key]) => !metricsToShow.includes(key),
  );
  const maxMetric = filteredMetrics.reduce((max, current) => {
    if (Number(current[1]) > Number(max[1])) {
      return current;
    }
    return max;
  }, filteredMetrics[0])[0];
  let color = "";
  switch (maxMetric) {
    case "major_updates":
      color = "text-red-500";
      break;
    case "minor_updates":
      color = "text-yellow-500";
      break;
    default:
      color = "text-blue-500";
  }
  return <CircleArrowUp className={`size-6 shrink-0 ${color}`} />;
}
