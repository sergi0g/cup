import { useEffect, useState } from "react";
import type { Data } from "../types";

export const useData = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [isError, setIsError] = useState(false);
  const [data, setData] = useState<Data | null>(null);

  useEffect(() => {
    if (isLoading || isError || !!data) return;
    setIsLoading(true);
    setIsError(false);
    setData(null);
    fetch(
      process.env.NODE_ENV === "production"
        ? "./api/v3/json"
        : `http://${window.location.hostname}:8000/api/v3/json`,
    )
      .then((response) => {
        if (response.ok) return response.json();
        throw new Error("Failed to fetch data");
      })
      .then((data) => {
        setData(data as Data);
      })
      .catch((error: unknown) => {
        setIsError(true);
        console.error(error);
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [data, isError, isLoading]);

  return {
    data,
    isLoading,
    isError,
  };
};
