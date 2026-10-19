import { describe, expect, it } from "vitest";
import { pickMood, type MoodInputs } from "./pickMood";

const BASE: MoodInputs = {
  notification: false,
  pulse: null,
  timerFinished: false,
  timerRunning: false,
  timerPhase: null,
  fullscreenApp: false,
  mediaPlaying: false,
  userIdle: false,
  night: false,
};

describe("pickMood", () => {
  it("defaults to idle", () => expect(pickMood(BASE)).toBe("idle"));
  it("alerts on notifications before anything else", () =>
    expect(pickMood({ ...BASE, notification: true, pulse: "happy", fullscreenApp: true })).toBe("alert"));
  it("shows one-off reactions over background states", () =>
    expect(pickMood({ ...BASE, pulse: "sad", mediaPlaying: true })).toBe("sad"));
  it("is happy when a timer phase finishes", () => expect(pickMood({ ...BASE, timerFinished: true })).toBe("happy"));
  it("focuses during a focus phase or a fullscreen app", () => {
    expect(pickMood({ ...BASE, timerRunning: true, timerPhase: "focus" })).toBe("focus");
    expect(pickMood({ ...BASE, fullscreenApp: true })).toBe("focus");
  });
  it("sleeps on a break, when idle, or at night", () => {
    expect(pickMood({ ...BASE, timerRunning: true, timerPhase: "shortBreak" })).toBe("sleepy");
    expect(pickMood({ ...BASE, userIdle: true })).toBe("sleepy");
    expect(pickMood({ ...BASE, night: true })).toBe("sleepy");
  });
  it("prefers music over sleeping at night", () => expect(pickMood({ ...BASE, mediaPlaying: true, night: true })).toBe("music"));
});
