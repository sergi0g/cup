import { ChangeEvent, useState } from "react";
import { theme } from "../theme";
import { IconSearch, IconX } from "@tabler/icons-react";

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
    <div className={`w-full px-6 text-${theme}-500`}>
      <div
        className={`flex items-center w-full rounded-md border border-${theme}-200 dark:border-${theme}-700 px-2 gap-1 bg-${theme}-800 flex-nowrap peer`}
      >
        <IconSearch className="size-5" />
        <div className="w-full">
          <input
            className={`w-full h-10 text-sm text-${theme}-400 focus:outline-none peer bg-transparent placeholder:text-${theme}-500`}
            placeholder="Search"
            onChange={handleChange}
            value={searchQuery}
          ></input>
        </div>
        {showClear && (
          <button onClick={handleClear} className={`hover:text-${theme}-400`}>
            <IconX className="size-5" />
          </button>
        )}
      </div>
      <div
        className="relative -translate-y-[8px] h-[8px] border-b-blue-600 border-b-2 w-0 peer-has-[:focus]:w-full transition-all duration-200 rounded-md left-1/2 -translate-x-1/2"
        style={{ clipPath: "inset(calc(100% - 2px) 0 0 0)" }}
      ></div>
    </div>
  );
}
