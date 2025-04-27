import { Provider, Root, Trigger, Content } from "@radix-ui/react-tooltip";
import { cn } from "../../utils";
import { forwardRef, ReactNode } from "react";
import { theme } from "../../theme";

const TooltipContent = forwardRef<
  React.ElementRef<typeof Content>,
  React.ComponentPropsWithoutRef<typeof Content>
>(({ className, sideOffset = 4, ...props }, ref) => (
  <Content
    ref={ref}
    sideOffset={sideOffset}
    className={cn(
      `z-50 overflow-hidden rounded-md border border-${theme}-200 dark:border-${theme}-800 bg-white px-3 py-1.5 text-sm text-${theme}-950 shadow-md animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 dark:border-${theme}-800 dark:bg-${theme}-950 dark:text-${theme}-50`,
      className,
    )}
    {...props}
  />
));

TooltipContent.displayName = Content.displayName;

const WithTooltip = ({
  children,
  text,
  className,
}: {
  children: ReactNode;
  text: string;
  className?: string;
}) => {
  return (
    <Provider>
      <Root>
        <Trigger className={className} asChild>
          {children}
        </Trigger>
        <TooltipContent>
          <p className="text-black dark:text-white">{text}</p>
        </TooltipContent>
      </Root>
    </Provider>
  );
};

export { WithTooltip };
