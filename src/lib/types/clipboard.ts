export type ClipboardFilter = "all" | "text" | "image" | "link" | "pinned";

export type ClipboardContentType = "text" | "url" | "color" | "code" | "image";

export interface ClipboardItem {
  id: number;
  contentType: ClipboardContentType;
  textContent: string | null;
  imagePath: string | null;
  thumbnailBase64: string | null;
  sourceAppName: string | null;
  isPinned: boolean;
  isSensitive: boolean;
  createdAt: number;
}
