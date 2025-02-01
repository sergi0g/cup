import Link from "next/link";
import { ReactNode } from "react";
import { twMerge } from "tailwind-merge";

interface ButtonProps {
  href: string;
  className?: string;
  children: ReactNode;
}

export default function Button({ href, className, children }: ButtonProps) {
  return (
    <Link
      href={href}
      className={twMerge(
        "flex items-center justify-center rounded-md border border-transparent px-8 py-3 text-base font-medium no-underline transition-colors duration-200 md:text-lg md:leading-6",
        className,
      )}
    >
      {children}
    </Link>
  );
}
