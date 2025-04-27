import { FilterIcon } from "lucide-react";
import { Filters as FiltersType } from "../types";
import { cn } from "../utils";
import { Popover, PopoverTrigger, PopoverContent } from "./ui/Popover";
import { Checkbox } from "./ui/Checkbox";

interface Props {
  filters: FiltersType;
  setFilters: (filters: FiltersType) => void;
}

export default function Filters({ filters, setFilters }: Props) {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <button className="ml-auto">
          <FilterIcon
            size={24}
            className={cn("text-current", filters.onlyInUse && "fill-current")}
          />
        </button>
      </PopoverTrigger>
      <PopoverContent className="mr-3 flex w-fit flex-col gap-2 p-4">
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
            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Show only images in use
          </label>
        </div>
      </PopoverContent>
    </Popover>
  );
}
