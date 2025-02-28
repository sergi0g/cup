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
    <div className="p-4 bg-white dark:bg-black group">
      <Icon className="text-black size-7 group-hover:size-9 dark:text-white inline mr-2 transition-[width,height] duration-200" />
      <span className="align-middle text-2xl font-bold text-black dark:text-white">
        {name}
      </span>
      <p className="text-xl font-semibold text-neutral-500 dark:text-neutral-500">
        {description}
      </p>
    </div>
  );
}
