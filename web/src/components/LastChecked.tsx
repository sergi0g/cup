import { intlFormatDistance } from "date-fns/intlFormatDistance";
import { theme } from "../theme";

export function LastChecked({ datetime }: { datetime: string }) {
  const date = intlFormatDistance(new Date(datetime), new Date());
  return (
    <h3 className={`text-${theme}-600 dark:text-${theme}-500`}>
      Last checked {date}
    </h3>
  );
}
