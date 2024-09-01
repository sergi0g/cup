import {
  IconCircleArrowUpFilled,
  IconCircleCheckFilled,
  IconCube,
  IconHelpCircleFilled,
} from "@tabler/icons-react";

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
        <IconCircleCheckFilled className="text-green-500 ml-auto size-6 shrink-0" />
      )}
      {status == true && (
        <IconCircleArrowUpFilled className="text-blue-500 ml-auto size-6 shrink-0" />
      )}
      {status == null && (
        <IconHelpCircleFilled className="text-gray-500 ml-auto size-6 shrink-0" />
      )}
    </li>
  );
}
