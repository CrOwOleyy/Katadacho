export interface CleanResult {
  mode: "quick" | "deep";
  processesTrimmed: number;
  freedBytes: number;
  usedBeforeBytes: number;
  usedAfterBytes: number;
  failedSteps: string[];
}
