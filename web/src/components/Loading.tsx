import { IconLoader2 } from "@tabler/icons-react";
import { Data } from "../types";
import Logo from "./Logo";

export default function Loading({ onLoad }: { onLoad: (data: Data) => void }) {
  const theme = "neutral";
  fetch(
    process.env.NODE_ENV === "production"
      ? "/json"
      : `http://${window.location.hostname}:8000/json`,
  ).then((response) => response.json().then((data) => onLoad(data)));
  return (
    <div
      className={`flex justify-center min-h-screen bg-${theme}-50 dark:bg-${theme}-950`}
    >
      <div className="lg:px-8 sm:px-6 px-4 max-w-[80rem] mx-auto h-full w-full absolute overflow-hidden">
        <div className="flex flex-col max-w-[48rem] mx-auto h-full my-8">
          <div className="flex items-center gap-1">
            <h1 className="text-5xl lg:text-6xl font-bold dark:text-white">
              Cup
            </h1>
            <Logo />
          </div>
          <div
            className={`h-full flex justify-center
            items-center gap-1 text-${theme}-500 dark:text-${theme}-400`}
          >
            Loading <IconLoader2 className="animate-spin" />
          </div>
        </div>
      </div>
    </div>
  );
}
