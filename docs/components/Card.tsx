import { ReactNode, createElement } from "react";

export function Card({
  name,
  icon,
  description,
}: {
  name: string;
  icon: ReactNode;
  description: string;
}) {
  const iconElement = createElement(icon, {
    className: "text-black size-7 dark:text-white inline mr-2",
  });
  return (
    <div>
      {iconElement}
      <span className="align-middle text-2xl font-bold text-black dark:text-white">
        {name}
      </span>
      <p className="text-2xl font-semibold text-neutral-500 dark:text-neutral-500">
        {description}
      </p>
    </div>
  );
}
