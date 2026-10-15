export interface NotificationToast {
  id: number;
  appName: string;
  title: string;
  body: string;
  createdMs: number;
}

export type NotificationAccess = "allowed" | "denied" | "unspecified";
