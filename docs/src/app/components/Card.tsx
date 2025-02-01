import { Icon as IconType } from "@tabler/icons-react";

export function Card({
  name,
  icon: Icon,
  description,
}: {
  name: string;
  icon: IconType;
  description: string;
}) {
  return (
    <div>
      <Icon className="text-black size-7 dark:text-white inline mr-2" />
      <span className="align-middle text-2xl font-bold text-black dark:text-white">
        {name}
      </span>
      <p className="text-2xl font-semibold text-neutral-500 dark:text-neutral-500">
        {description}
      </p>
    </div>
  );
}
