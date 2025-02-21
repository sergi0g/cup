export interface Data {
  metrics: {
    monitored_images: number;
    up_to_date: number;
    updates_available: number;
    major_updates: number;
    minor_updates: number;
    patch_updates: number;
    other_updates: number;
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
  result: {
    has_update: boolean | null;
    info: VersionInfo | DigestInfo | null;
    error: string | null;
  };
  time: number;
  server: string | null;
}

interface VersionInfo {
  type: "version";
  version_update_type: "major" | "minor" | "patch";
  new_tag: string;
  current_version: string;
  new_version: string;
}

interface DigestInfo {
  type: "digest";
  local_digests: string[];
  remote_digest: string;
}
