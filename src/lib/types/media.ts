export interface MediaInfo {
  title: string;
  artist: string;
  playing: boolean;
  coverBase64: string | null;
  coverMime: string | null;
}

export interface MediaPosition {
  positionSeconds: number;
  durationSeconds: number | null;
  playing: boolean;
}
