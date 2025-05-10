import { useState } from "react";
import Logo from "./components/Logo";
import Statistic from "./components/Statistic";
import Image from "./components/Image";
import { LastChecked } from "./components/LastChecked";
import Loading from "./components/Loading";
import { Filters as FiltersType } from "./types";
import { theme } from "./theme";
import RefreshButton from "./components/RefreshButton";
import Search from "./components/Search";
import { Server } from "./components/Server";
import { useData } from "./hooks/use-data";
import DataLoadingError from "./components/DataLoadingError";
import Filters from "./components/Filters";
import { Filter, FilterX } from "lucide-react";
import { WithTooltip } from "./components/ui/Tooltip";
import { getDescription } from "./utils";

const SORT_ORDER = [
  "monitored_images",
  "updates_available",
  "major_updates",
  "minor_updates",
  "patch_updates",
  "other_updates",
  "up_to_date",
  "unknown",
];

function App() {
  const { data, isLoading, isError } = useData();

  const [showFilters, setShowFilters] = useState<boolean>(false);
  const [filters, setFilters] = useState<FiltersType>({
    onlyInUse: false,
    registries: [],
    statuses: [],
  });
  const [searchQuery, setSearchQuery] = useState("");

  if (isLoading) return <Loading />;
  if (isError || !data) return <DataLoadingError />;
  const toggleShowFilters = () => {
    if (showFilters) {
      setFilters({ onlyInUse: false, registries: [], statuses: [] });
    }
    setShowFilters(!showFilters);
  };

  return (
    <div
      className={`flex min-h-screen justify-center bg-white dark:bg-${theme}-950`}
    >
      <div className="mx-auto h-full w-full max-w-[80rem] px-4 sm:px-6 lg:px-8">
        <div className="mx-auto my-8 flex h-full max-w-[48rem] flex-col">
          <div className="flex items-center gap-1">
            <h1 className="text-5xl font-bold tracking-tight lg:text-6xl dark:text-white">
              Cup
            </h1>
            <Logo />
          </div>
          <div
            className={`border shadow-sm border-${theme}-200 dark:border-${theme}-900 my-8 rounded-md`}
          >
            <dl className="grid grid-cols-2 gap-1 overflow-hidden *:relative lg:grid-cols-4">
              {Object.entries(data.metrics)
                .sort((a, b) => {
                  return SORT_ORDER.indexOf(a[0]) - SORT_ORDER.indexOf(b[0]);
                })
                .map(([name]) => (
                  <Statistic
                    name={name as keyof typeof data.metrics}
                    metrics={data.metrics}
                    key={name}
                  />
                ))}
            </dl>
          </div>
          <div
            className={`border shadow-sm border-${theme}-200 dark:border-${theme}-900 my-8 rounded-md`}
          >
            <div
              className={`flex items-center justify-between gap-3 px-6 py-4 text-${theme}-500`}
            >
              <LastChecked datetime={data.last_updated} />
              <div className="flex gap-3">
                <WithTooltip
                  text={showFilters ? "Clear filters" : "Show filters"}
                >
                  <button onClick={toggleShowFilters}>
                    {showFilters ? <FilterX /> : <Filter />}
                  </button>
                </WithTooltip>
                <RefreshButton />
              </div>
            </div>
            <div className="flex gap-2 px-6 text-black dark:text-white">
              <Search onChange={setSearchQuery} />
            </div>
            {showFilters && (
              <Filters
                filters={filters}
                setFilters={setFilters}
                registries={[
                  ...new Set(data.images.map((image) => image.parts.registry)),
                ]}
              />
            )}
            <ul>
              {Object.entries(
                data.images.reduce<Record<string, typeof data.images>>(
                  (acc, image) => {
                    const server = image.server ?? "";
                    if (!Object.hasOwn(acc, server)) acc[server] = [];
                    acc[server].push(image);
                    return acc;
                  },
                  {},
                ),
              )
                .sort()
                .map(([server, images]) => (
                  <Server name={server} key={server}>
                    {images
                      .filter((image) =>
                        filters.onlyInUse ? image.used_by.length > 0 : true,
                      )
                      .filter((image) =>
                        filters.registries.length == 0
                          ? true
                          : filters.registries.includes(image.parts.registry),
                      )
                      .filter((image) =>
                        filters.statuses.length == 0
                          ? true
                          : filters.statuses.includes(getDescription(image)),
                      )
                      .filter((image) => image.reference.includes(searchQuery))
                      .map((image) => (
                        <Image data={image} key={image.reference} />
                      ))}
                  </Server>
                ))}
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
