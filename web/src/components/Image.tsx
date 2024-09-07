import {
  IconCircleArrowUpFilled,
  IconCircleCheckFilled,
  IconCube,
  IconHelpCircleFilled,
} from "@tabler/icons-react";
import { WithTooltip } from "./Tooltip";

export default function Image({
  name,
  status,
}: {
  name: string;
  status: boolean | null;
}) {
  return (
    <li className="break-all">
      <IconCube className="size-6 shrink-0" />
      {name}
      {status == false && (
        <WithTooltip
          text="Up to date"
          className="text-green-500 ml-auto size-6 shrink-0"
        >
          <IconCircleCheckFilled />
        </WithTooltip>
      )}
      {status == true && (
        <WithTooltip
          text="Update available"
          className="text-blue-500 ml-auto size-6 shrink-0"
        >
          <IconCircleArrowUpFilled />
        </WithTooltip>
      )}
      {status == null && (
        <WithTooltip
          text="Unknown"
          className="text-gray-500 ml-auto size-6 shrink-0"
        >
          <IconHelpCircleFilled />
        </WithTooltip>
      )}
    </li>
  );
}
