import { MouseEvent, useState } from "react";
import Logo from "./components/Logo";
import Statistic from "./components/Statistic";
import Image from "./components/Image";
import { LastChecked } from "./components/LastChecked";
import Loading from "./components/Loading";
import { Data } from "./types";

function App() {
  const [data, setData] = useState<Data | null>(null);
  const theme = "neutral"; // Stupid, I know but I want both the dev and prod to work easily.
  if (!data) return <Loading onLoad={setData} />;
  const refresh = (event: MouseEvent) => {
    const btn = event.currentTarget as HTMLButtonElement;
    btn.disabled = true;

    let request = new XMLHttpRequest();
    request.onload = () => {
      if (request.status === 200) {
        window.location.reload();
      }
    };
    request.open(
      "GET",
      process.env.NODE_ENV === "production"
        ? "/refresh"
        : `http://${window.location.hostname}:8000/refresh`
    );
    request.send();
  };
  return (
    <div
      className={`flex justify-center min-h-screen bg-${theme}-50 dark:bg-${theme}-950`}
    >
      <div className="lg:px-8 sm:px-6 px-4 max-w-[80rem] mx-auto h-full w-full">
        <div className="flex flex-col max-w-[48rem] mx-auto h-full my-8">
          <div className="flex items-center gap-1">
            <h1 className="text-5xl lg:text-6xl font-bold dark:text-white">
              Cup
            </h1>
            <Logo />
          </div>
          <div
            className={`shadow-sm bg-white dark:bg-${theme}-900 rounded-md my-8`}
          >
            <dl className="lg:grid-cols-4 md:grid-cols-2 gap-1 grid-cols-1 grid overflow-hidden *:relative">
              {Object.entries(data.metrics).map(([name, value]) => (
                <Statistic name={name} value={value} key={name} />
              ))}
            </dl>
          </div>
          <div
            className={`shadow-sm bg-white dark:bg-${theme}-900 rounded-md my-8`}
          >
            <div
              className={`flex justify-between items-center px-6 py-4 text-${theme}-500`}
            >
              <LastChecked datetime={data.last_updated} />
              <button className="group" onClick={refresh}>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  className="group-disabled:animate-spin"
                >
                  <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                  <path d="M4,11A8.1,8.1 0 0 1 19.5,9M20,5v4h-4" />
                  <path d="M20,13A8.1,8.1 0 0 1 4.5,15M4,19v-4h4" />
                </svg>
              </button>
            </div>
            <ul
              className={`*:py-4 *:px-6 *:flex *:items-center *:gap-3 dark:divide-${theme}-800 divide-y dark:text-white`}
            >
              {Object.entries(data.images).map(([name, status]) => (
                <Image name={name} status={status} key={name} />
              ))}
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
