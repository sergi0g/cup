import { Data } from "../types";
import Logo from "./Logo";
import { theme } from "../theme";
import { RefreshCw } from "lucide-react";

export default function Loading({ onLoad }: { onLoad: (data: Data) => void }) {
  fetch(
    process.env.NODE_ENV === "production"
      ? "/api/v3/json"
      : `http://${window.location.hostname}:8000/api/v3/json`,
  ).then((response) =>
    response.json().then((data) => {
      onLoad(data as Data);
    }),
  );
  return (
    <div
      className={`flex min-h-screen justify-center bg-${theme}-50 dark:bg-${theme}-950`}
    >
      <div className="absolute mx-auto h-full w-full max-w-[80rem] overflow-hidden px-4 sm:px-6 lg:px-8">
        <div className="mx-auto my-8 flex h-full max-w-[48rem] flex-col">
          <div className="flex items-center gap-1">
            <h1 className="text-5xl font-bold lg:text-6xl dark:text-white">
              Cup
            </h1>
            <Logo />
          </div>
          <div
            className={`flex h-full items-center justify-center gap-1 text-${theme}-500 dark:text-${theme}-400`}
          >
            Loading <RefreshCw className="animate-spin" />
          </div>
        </div>
      </div>
    </div>
  );
}
