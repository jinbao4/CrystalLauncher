export type Account = {
  uuid: string;
  name: string;
  mc_token: string;
  refresh_token: string;
  expires_at: number;
};

export interface VersionEntry {
  id: string;
  type: string;
  url: string;
  time: string;
  releaseTime: string;
}

export interface VersionManifest {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: VersionEntry[];
}
