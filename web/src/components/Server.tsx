import {
  Disclosure,
  DisclosureButton,
  DisclosurePanel,
} from "@headlessui/react";
import { theme } from "../theme";
import { ChevronDown } from "lucide-react";

export function Server({
  name,
  children,
}: {
  name: string;
  children: React.ReactNode;
}) {
  if (name.length === 0)
    return (
      <li className="mb-8 last:mb-0">
        <ul className={`dark:divide-${theme}-900 divide-y dark:text-white`}>
          {children}
        </ul>
      </li>
    );
  return (
    <Disclosure defaultOpen as="li" className={`mb-4 last:mb-0`}>
      <DisclosureButton className="group my-4 flex w-full items-center justify-between px-6">
        <span
          className={`text-lg font-semibold text-${theme}-600 dark:text-${theme}-400 group-data-[hover]:text-${theme}-800 group-data-[hover]:dark:text-${theme}-200 transition-colors duration-300`}
        >
          {name}
        </span>
        <ChevronDown
          className={`size-5 duration-300 text-${theme}-600 transition-transform group-data-[open]:rotate-180 dark:text-${theme}-400 group-data-[hover]:text-${theme}-800 group-data-[hover]:dark:text-${theme}-200 transition-colors`}
        />
      </DisclosureButton>
      <DisclosurePanel
        className={`dark:divide-${theme}-900 divide-y dark:text-white`}
        as="ul"
        transition
      >
        {children}
      </DisclosurePanel>
    </Disclosure>
  );
}
