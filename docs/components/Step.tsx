import React, { ReactNode } from "react";

export function Step({
  children,
  title,
  number,
}: {
  children: ReactNode;
  title: string;
  number: number;
}) {
  return (
    <div className="mb-2 flex grow-0 items-baseline">
      <p className="m-2 flex size-10 items-center justify-center rounded-full bg-neutral-100 dark:bg-neutral-900">
        {number}
      </p>
      <div>
        <p className="text-xl font-bold">{title}</p>
        <div className="my-6">{children}</div>
      </div>
    </div>
  );
}
