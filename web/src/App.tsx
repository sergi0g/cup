import { useState } from "react";
import Logo from "./components/Logo";
import Statistic from "./components/Statistic";
import Image from "./components/Image";
import { LastChecked } from "./components/LastChecked";
import Loading from "./components/Loading";
import { Data } from "./types";
import { theme } from "./theme";
import RefreshButton from "./components/RefreshButton";
import Search from "./components/Search";

function App() {
  const [data, setData] = useState<Data | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  if (!data) return <Loading onLoad={setData} />;
  return (
    <div
      className={`flex min-h-screen justify-center bg-${theme}-50 dark:bg-${theme}-950`}
    >
      <div className="mx-auto h-full w-full max-w-[80rem] px-4 sm:px-6 lg:px-8">
        <div className="mx-auto my-8 flex h-full max-w-[48rem] flex-col">
          <div className="flex items-center gap-1">
            <h1 className="text-5xl font-bold lg:text-6xl dark:text-white">
              Cup
            </h1>
            <Logo />
          </div>
          <div
            className={`bg-white shadow-sm dark:bg-${theme}-900 my-8 rounded-md`}
          >
            <dl className="grid grid-cols-1 gap-1 overflow-hidden *:relative md:grid-cols-2 lg:grid-cols-4">
              {Object.entries(data.metrics).map(([name, value]) => (
                <Statistic name={name} value={value} key={name} />
              ))}
            </dl>
          </div>
          <div
            className={`bg-white shadow-sm dark:bg-${theme}-900 my-8 rounded-md`}
          >
            <div
              className={`flex items-center justify-between px-6 py-4 text-${theme}-500`}
            >
              <LastChecked datetime={data.last_updated} />
              <RefreshButton />
            </div>
            <Search onChange={setSearchQuery} />
            <ul className={`dark:divide-${theme}-800 divide-y dark:text-white`}>
              {data.images
                .filter((image) => image.reference.includes(searchQuery))
                .map((image) => (
                  <Image data={image} key={image.reference} />
                ))}
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
