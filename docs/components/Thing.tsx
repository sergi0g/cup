import React, { ReactNode, createElement } from "react";

export function Thing({
  title,
  icon,
  content,
}: {
  title: string;
  icon: ReactNode;
  content: string;
}) {
  const iconElement = createElement(icon, {
    className: "text-black size-7 dark:text-white inline mr-2",
  });
  return (
    <div>
      {iconElement}
      <span className="align-middle text-2xl font-bold text-black dark:text-white">
        {title}
      </span>
      <p className="text-2xl font-semibold text-neutral-500 dark:text-neutral-500">
        {content}
      </p>
    </div>
  );
}
