import { useId } from "react";

const SIZE = 36;

export function GridPattern() {
  const id = useId();

  return (
    <svg
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 bottom-0 left-0 right-0 top-0 h-full w-full -z-10 bg-white stroke-neutral-200 dark:stroke-white/10 dark:bg-black"
    >
      <defs>
        <pattern
          id={id}
          width={SIZE}
          height={SIZE}
          patternUnits="userSpaceOnUse"
          x={-1}
          y={-1}
        >
          <path
            d={`M.5 ${SIZE}V.5H${SIZE}`}
            fill="none"
          />
        </pattern>
      </defs>
      <rect width="100%" height="100%" strokeWidth={0} fill={`url(#${id})`} />
    </svg>
  );
}
