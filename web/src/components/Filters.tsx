import { theme } from "../theme";
import { Filters as FiltersType } from "../types";
import { Checkbox } from "./ui/Checkbox";

interface Props {
  filters: FiltersType;
  setFilters: (filters: FiltersType) => void;
}

export default function Filters({ filters, setFilters }: Props) {
  return (
      <div className="flex w-fit flex-col gap-2 px-6 py-4">
        <div className="ml-auto flex items-center space-x-2">
          <Checkbox
            id="inUse"
            checked={filters.onlyInUse}
            onCheckedChange={(value) => {
              setFilters({
                ...filters,
                onlyInUse: value === "indeterminate" ? false : value,
              });
            }}
          />
          <label
            htmlFor="inUse"
            className={`text-sm font-medium leading-none text-${theme}-600 dark:text-${theme}-400 hover:text-black dark:hover:text-white peer-hover:text-black peer-hover:dark:text-white peer-data-[state=checked]:text-black dark:peer-data-[state=checked]:text-white transition-colors duration-200`}
          >
            Hide unused images
          </label>
        </div>
      </div>
  );
}
