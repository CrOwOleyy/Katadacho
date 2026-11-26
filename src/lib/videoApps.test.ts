import { describe, expect, it } from "vitest";
import { isVideoApp } from "./videoApps";

describe("isVideoApp", () => {
  it("recognises video players by their executable", () => {
    expect(isVideoApp({ process: "VLC.exe", title: "film.mkv" })).toBe(true);
    expect(isVideoApp({ process: "mpv.exe", title: "" })).toBe(true);
  });

  it("recognises streaming sites in a browser title", () => {
    expect(isVideoApp({ process: "chrome.exe", title: "Épisode 3 - Crunchyroll - Google Chrome" })).toBe(true);
    expect(isVideoApp({ process: "firefox.exe", title: "Titre de la vidéo - YouTube - Mozilla Firefox" })).toBe(true);
    expect(isVideoApp({ process: "msedge.exe", title: "Disney+ - Microsoft Edge" })).toBe(true);
  });

  it("ignores ordinary windows", () => {
    expect(isVideoApp({ process: "Code.exe", title: "main.rs - Visual Studio Code" })).toBe(false);
    expect(isVideoApp({ process: "explorer.exe", title: "Documents" })).toBe(false);
  });
});
