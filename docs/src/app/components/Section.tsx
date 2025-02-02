import { ReactNode } from "react";
import { GradientText } from "./GradientText";

export function Section({
  title,
  className,
  children,
}: {
  title: string;
  className: string;
  children: ReactNode;
}) {
  return (
    <div className="border-t border-t-neutral-300 bg-neutral-50 py-32 dark:border-t-neutral-600/30 dark:bg-neutral-950">
      <div className="mx-auto w-full max-w-screen-xl">
        <GradientText
          text={title}
          className="mx-auto mb-20 w-fit text-center text-4xl font-bold tracking-tighter"
          innerClassName={className}
          blur={12}
        />
        <div className="m-2 grid w-full auto-cols-fr gap-20 lg:grid-cols-3">
          {children}
        </div>
      </div>
    </div>
  );
}
