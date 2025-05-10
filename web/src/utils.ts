import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { Image } from "./types";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getDescription(image: Image) {
  switch (image.result.has_update) {
    case null:
      return "Unknown";
    case false:
      return "Up to date";
    case true:
      if (image.result.info?.type === "version") {
        switch (image.result.info.version_update_type) {
          case "major":
            return "Major update";
          case "minor":
            return "Minor update";
          case "patch":
            return "Patch update";
        }
      } else if (image.result.info?.type === "digest") {
        return "Digest update";
      }
      return "Unknown";
  }
}

export function truncateArray(arr: string[]) {
  if (arr.length > 1) {
    return `${arr[0]} +${(arr.length - 1).toString()} more`
  } else if (arr.length == 1) {
    return arr[0]
  }
}