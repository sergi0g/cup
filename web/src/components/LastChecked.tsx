import { intlFormatDistance } from "date-fns/intlFormatDistance";

export function LastChecked({ datetime }: { datetime: string }) {
  const date = intlFormatDistance(new Date(datetime), new Date());
  return <h3>Last checked {date}</h3>;
}
