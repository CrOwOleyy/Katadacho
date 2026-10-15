export type LauncherKind = "app" | "uwp" | "file" | "folder";

export interface LauncherResult {
  name: string;
  path: string;
  kind: LauncherKind;
  score: number;
}
