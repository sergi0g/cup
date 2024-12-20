import React from "react";
import { clsx } from "clsx";

export function GradientText({
  text,
  innerClassName,
  className,
  blur,
}: {
  text: string;
  innerClassName: string;
  className?: string;
  blur: number;
}) {
  return (
    <div className={clsx("relative", className)}>
      <p className={clsx("bg-clip-text text-transparent", innerClassName)}>
        {text}
      </p>
      <p
        className={clsx(
          "pointer-events-none absolute top-0 hidden select-none bg-clip-text text-transparent dark:block",
          innerClassName,
        )}
        style={{ filter: `blur(${blur}px)` }}
      >
        {text}
      </p>
    </div>
  );
}
