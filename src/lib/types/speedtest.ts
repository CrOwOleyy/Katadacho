export type SpeedtestEvent =
  | { stage: "ping"; latencyMs: number; jitterMs: number }
  | { stage: "download"; mbps: number; progress: number }
  | { stage: "upload"; mbps: number; progress: number }
  | { stage: "done"; downloadMbps: number; uploadMbps: number };
