import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const getLauncherIcon = vi.fn<(path: string) => Promise<string>>();
vi.mock("../lib/ipc", () => ({ ipc: { getLauncherIcon: (path: string) => getLauncherIcon(path) } }));

const { loadIcon, resetIconCache, RETRY_AFTER_MS } = await import("./useAppIcon");

describe("loadIcon", () => {
  beforeEach(() => {
    resetIconCache();
    getLauncherIcon.mockReset();
    vi.useFakeTimers();
  });
  afterEach(() => vi.useRealTimers());

  it("shares one request between simultaneous callers", async () => {
    getLauncherIcon.mockResolvedValue("PNG");
    const [first, second] = await Promise.all([loadIcon("firefox.exe"), loadIcon("firefox.exe")]);
    expect([first, second]).toEqual(["PNG", "PNG"]);
    expect(getLauncherIcon).toHaveBeenCalledTimes(1);
  });

  it("keeps a found icon without asking again", async () => {
    getLauncherIcon.mockResolvedValue("PNG");
    await loadIcon("steam.exe");
    await loadIcon("steam.exe");
    expect(getLauncherIcon).toHaveBeenCalledTimes(1);
  });

  it("retries a failed icon only after the delay", async () => {
    getLauncherIcon.mockRejectedValueOnce(new Error("occupé")).mockResolvedValue("PNG");
    expect(await loadIcon("firefox.exe")).toBeNull();
    expect(await loadIcon("firefox.exe")).toBeNull();
    expect(getLauncherIcon).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(RETRY_AFTER_MS);
    expect(await loadIcon("firefox.exe")).toBe("PNG");
  });

  it("treats an empty answer as a missing icon", async () => {
    getLauncherIcon.mockResolvedValue("");
    expect(await loadIcon("vide.exe")).toBeNull();
  });
});
