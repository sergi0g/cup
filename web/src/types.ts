export type Data = {
  metrics: {
    monitored_images: number;
    up_to_date: number;
    update_available: number;
    unknown: number;
  };
  images: {
    [key: string]: boolean | null;
  };
  last_updated: string;
};
