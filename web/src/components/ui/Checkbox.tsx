"use client";

import * as React from "react";
import * as CheckboxPrimitive from "@radix-ui/react-checkbox";
import { Check } from "lucide-react";
import { cn } from "../../utils";
import { theme } from "../../theme";

const Checkbox = React.forwardRef<
  React.ElementRef<typeof CheckboxPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof CheckboxPrimitive.Root>
>(({ className, ...props }, ref) => (
  <CheckboxPrimitive.Root
    ref={ref}
    className={cn(
      `border-${theme}-600 dark:border-${theme}-400 group peer h-4 w-4 shrink-0 rounded-sm border shadow transition-colors duration-200 hover:border-black focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-blue-500 data-[state=checked]:border-0 data-[state=checked]:bg-blue-500 data-[state=checked]:text-white hover:data-[state=checked]:bg-blue-600 dark:hover:border-white dark:hover:data-[state=checked]:bg-blue-400`,
      className,
    )}
    {...props}
  >
    <CheckboxPrimitive.Indicator
      className={cn("flex items-center justify-center text-current")}
    >
      <Check
        className={`h-3 w-3 group-data-[state=checked]:text-white dark:group-data-[state=checked]:text-${theme}-950`}
        strokeWidth={3}
      />
    </CheckboxPrimitive.Indicator>
  </CheckboxPrimitive.Root>
));
Checkbox.displayName = CheckboxPrimitive.Root.displayName;

export { Checkbox };
