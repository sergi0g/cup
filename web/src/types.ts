export interface Data {
  metrics: {
    monitored_images: number;
    up_to_date: number;
    update_available: number;
    unknown: number;
  };
  images: Image[];
  last_updated: string;
}

export interface Image {
  reference: string;
  parts: {
    registry: string;
    repository: string;
    tag: string;
  };
  local_digests: string[];
  remote_digest: string;
  result: {
    has_update: boolean | null;
    error: string | null;
  };
  time: number;
}
