export type AiStatus = "done" | "waiting" | "error" | "info";

export interface AiNotification {
  title: string;
  summary: string;
  status: AiStatus | string;
}

export interface AiQuestion {
  id: number;
  question: string;
  options: string[];
}

export interface ClaudeConfigStatus {
  path: string;
  exists: boolean;
  installed: boolean;
  upToDate: boolean;
}
