import { useState } from "react";
import { theme } from "../theme";
import { Filters as FiltersType } from "../types";
import { Checkbox } from "./ui/Checkbox";
import Select from "./ui/Select";
import { Server } from "lucide-react";

interface Props {
  filters: FiltersType;
  setFilters: (filters: FiltersType) => void;
  registries: string[];
}

const STATUSES = [
  "Major update",
  "Minor update",
  "Patch update",
  "Digest update",
  "Up to date",
  "Unknown",
];

export default function Filters({ filters, setFilters, registries }: Props) {
  const [selectedRegistries, setSelectedRegistries] = useState<
    FiltersType["registries"]
  >([]);
  const [selectedStatuses, setSelectedStatuses] = useState<
    FiltersType["statuses"]
  >([]);
  const handleSelectRegistries = (registries: string[]) => {
    setSelectedRegistries(registries);
    setFilters({
      ...filters,
      registries,
    });
  };
  const handleSelectStatuses = (statuses: string[]) => {
    if (statuses.every((status) => STATUSES.includes(status))) {
      setSelectedStatuses(statuses as FiltersType["statuses"]);
      setFilters({
        ...filters,
        statuses: statuses as FiltersType["statuses"],
      });
    }
  };
  return (
    <div className="flex w-full flex-col gap-4 px-6 py-4 sm:flex-row">
      <div className="flex items-center space-x-2">
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
          className={`text-sm font-medium leading-none text-${theme}-600 dark:text-${theme}-400 transition-colors duration-200 hover:text-black peer-hover:text-black peer-data-[state=checked]:text-black dark:hover:text-white peer-hover:dark:text-white dark:peer-data-[state=checked]:text-white`}
        >
          Hide unused images
        </label>
      </div>
      <Select
        Icon={Server}
        items={registries}
        placeholder="Registry"
        selectedItems={selectedRegistries}
        setSelectedItems={handleSelectRegistries}
      />
      <Select
        items={STATUSES}
        placeholder="Update type"
        selectedItems={selectedStatuses}
        setSelectedItems={handleSelectStatuses}
      />
    </div>
  );
}
