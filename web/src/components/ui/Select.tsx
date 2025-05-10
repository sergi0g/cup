import {
  Listbox,
  ListboxButton,
  ListboxOptions,
  ListboxOption,
} from "@headlessui/react";
import { ChevronDown, Check } from "lucide-react";
import { theme } from "../../theme";
import { cn, truncateArray } from "../../utils";
import { Server } from "lucide-react";

export default function Select({
  items,
  Icon,
  placeholder,
  selectedItems,
  setSelectedItems,
}: {
  items: string[];
  Icon?: typeof Server;
  placeholder: string;
  selectedItems: string[];
  setSelectedItems: (items: string[]) => void;
}) {
  return (
    <Listbox value={selectedItems} onChange={setSelectedItems} multiple>
      <div className="relative">
        <ListboxButton
          className={cn(
            `flex w-full gap-2 overflow-x-hidden rounded-md bg-${theme}-100 dark:bg-${theme}-900 border border-${theme}-200 dark:border-${theme}-700 group relative items-center py-1.5 pl-3 pr-2 text-left transition-colors duration-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-blue-500 sm:text-sm/6`,
            selectedItems.length == 0
              ? `text-${theme}-600 dark:text-${theme}-400 hover:text-black hover:dark:text-white`
              : "text-black dark:text-white",
          )}
        >
          {Icon && (
            <Icon
              className={cn(
                "size-4 shrink-0",
                selectedItems.length == 0
                  ? `text-${theme}-600 dark:text-${theme}-400 hover:text-black hover:dark:text-white`
                  : "text-black dark:text-white",
              )}
            />
          )}
          <span className="truncate">
            {selectedItems.length == 0
              ? placeholder
              : truncateArray(selectedItems)}
          </span>

          <ChevronDown
            aria-hidden="true"
            className={`ml-auto size-5 shrink-0 self-center text-${theme}-600 dark:text-${theme}-400 transition-colors duration-200 group-hover:text-black sm:size-4 group-hover:dark:text-white`}
          />
          <div
            className="absolute -bottom-px left-1/2 h-full w-0 -translate-x-1/2 rounded-md border-b-2 border-b-blue-600 transition-all duration-200 group-data-[open]:w-[calc(100%+2px)]"
            style={{ clipPath: "inset(calc(100% - 2px) 0 0 0)" }}
          ></div>
        </ListboxButton>
        <ListboxOptions
          transition
          className={`absolute z-10 mt-1 max-h-56 w-max overflow-y-auto overflow-x-hidden rounded-md bg-${theme}-100 dark:bg-${theme}-900 border border-${theme}-200 dark:border-${theme}-700 text-base shadow-lg ring-1 ring-black/5 focus:outline-none data-[closed]:data-[leave]:opacity-0 data-[leave]:transition data-[leave]:duration-100 data-[leave]:ease-in sm:text-sm`}
        >
          {items.map((item) => (
            <ListboxOption
              key={item}
              value={item}
              className={`group relative cursor-pointer text-nowrap py-2 pl-3 pr-9 data-[focus]:outline-none text-${theme}-600 dark:text-${theme}-400 transition-colors duration-200 data-[focus]:bg-black/10 data-[focus]:text-black dark:data-[focus]:bg-white/10 data-[focus]:dark:text-white`}
            >
              {item}
              <span
                className={`absolute inset-y-0 right-2 flex items-center text-${theme}-600 dark:text-${theme}-400 group-[:not([data-selected])]:hidden group-data-[focus]:text-black group-data-[focus]:dark:text-white`}
              >
                <Check aria-hidden="true" className="size-4" />
              </span>
            </ListboxOption>
          ))}
        </ListboxOptions>
      </div>
    </Listbox>
  );
}
