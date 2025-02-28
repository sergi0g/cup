import { ChangeEvent, useState } from "react";
import { theme } from "../theme";
import { SearchIcon, X } from "lucide-react";

export default function Search({
  onChange,
}: {
  onChange: (value: string) => void;
}) {
  const [searchQuery, setSearchQuery] = useState("");
  const [showClear, setShowClear] = useState(false);
  const handleChange = (event: ChangeEvent) => {
    const value = (event.target as HTMLInputElement).value;
    setSearchQuery(value);
    onChange(value);
    if (value !== "") {
      setShowClear(true);
    } else setShowClear(false);
  };
  const handleClear = () => {
    setShowClear(false);
    setSearchQuery("");
    onChange("");
  };
  return (
    <div className={`w-full px-6 text-black dark:text-white`}>
      <div
        className={`flex w-full items-center rounded-md border border-${theme}-200 dark:border-${theme}-700 gap-1 px-2 bg-${theme}-100 dark:bg-${theme}-900 peer flex-nowrap`}
      >
        <SearchIcon className={`size-5 text-${theme}-600 dark:text-${theme}-400`} />
        <div className="w-full">
          <input
            className={`h-10 w-full text-sm text-${theme}-800 dark:text-${theme}-200 peer bg-transparent focus:outline-none placeholder:text-${theme}-600 placeholder:dark:text-${theme}-400`}
            placeholder="Search"
            onChange={handleChange}
            value={searchQuery}
          ></input>
        </div>
        {showClear && (
          <button
            onClick={handleClear}
            className={`hover:text-${theme}-600 dark:hover:text-${theme}-400 transition-colors duration-200`}
          >
            <X className="size-5" />
          </button>
        )}
      </div>
      <div
        className="relative left-1/2 h-[8px] w-0 -translate-x-1/2 -translate-y-[8px] rounded-md border-b-2 border-b-blue-600 transition-all duration-200 peer-has-[:focus]:w-full"
        style={{ clipPath: "inset(calc(100% - 2px) 0 0 0)" }}
      ></div>
    </div>
  );
}
