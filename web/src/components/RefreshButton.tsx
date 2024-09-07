import { MouseEvent } from "react";
import { WithTooltip } from "./Tooltip";

export default function RefreshButton() {
  const refresh = (event: MouseEvent) => {
    const btn = event.currentTarget as HTMLButtonElement;
    btn.disabled = true;

    const request = new XMLHttpRequest();
    request.onload = () => {
      if (request.status === 200) {
        window.location.reload();
      }
    };
    request.open(
      "GET",
      process.env.NODE_ENV === "production"
        ? "/refresh"
        : `http://${window.location.hostname}:8000/refresh`,
    );
    request.send();
  };
  return (
    <WithTooltip text="Reload">
      <button className="group" onClick={refresh}>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="group-disabled:animate-spin"
        >
          <path stroke="none" d="M0 0h24v24H0z" fill="none" />
          <path d="M4,11A8.1,8.1 0 0 1 19.5,9M20,5v4h-4" />
          <path d="M20,13A8.1,8.1 0 0 1 4.5,15M4,19v-4h4" />
        </svg>
      </button>
    </WithTooltip>
  );
}
