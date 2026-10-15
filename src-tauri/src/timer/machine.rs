//! Machine à états du minuteur et du Pomodoro, sans dépendance à Tauri : l'heure est passée
//! en paramètre, ce qui permet de la tester précisément.

use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    None,
    Countdown,
    Pomodoro,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
    Countdown,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Idle,
    Running,
    Paused,
    /// Phase suivante chargée, en attente d'un clic (quand l'enchaînement automatique est désactivé).
    Ready,
}

/// Durées en secondes, lues dans les réglages au début de chaque phase.
#[derive(Clone, Copy, Debug)]
pub struct Durations {
    pub focus: u32,
    pub short_break: u32,
    pub long_break: u32,
    pub sessions_before_long_break: u32,
}

impl Durations {
    fn for_phase(&self, phase: Phase) -> u32 {
        match phase {
            Phase::Focus => self.focus,
            Phase::ShortBreak => self.short_break,
            Phase::LongBreak => self.long_break,
            Phase::Countdown => 0,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub kind: Kind,
    pub phase: Option<Phase>,
    pub status: Status,
    pub remaining_seconds: u32,
    pub total_seconds: u32,
    pub completed_focus_sessions: u32,
    pub sessions_before_long_break: u32,
    /// Séances de concentration terminées dans le cycle en cours, pour afficher les pastilles.
    pub cycle_progress: u32,
}

/// Fin d'une phase, pour déclencher le son et l'animation.
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    pub phase: Phase,
    /// Phase chargée ensuite, `None` pour un minuteur simple.
    pub next_phase: Option<Phase>,
}

pub struct Machine {
    kind: Kind,
    phase: Option<Phase>,
    status: Status,
    total: Duration,
    /// Temps restant figé quand la machine n'est pas en cours d'exécution.
    remaining_frozen: Duration,
    deadline: Option<Instant>,
    completed_focus: u32,
    sessions_before_long_break: u32,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            kind: Kind::None,
            phase: None,
            status: Status::Idle,
            total: Duration::ZERO,
            remaining_frozen: Duration::ZERO,
            deadline: None,
            completed_focus: 0,
            sessions_before_long_break: 4,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    fn load_phase(&mut self, phase: Phase, seconds: u32) {
        self.phase = Some(phase);
        self.total = Duration::from_secs(u64::from(seconds));
        self.remaining_frozen = self.total;
        self.deadline = None;
        self.status = Status::Ready;
    }

    fn run_from(&mut self, now: Instant) {
        self.deadline = Some(now + self.remaining_frozen);
        self.status = Status::Running;
    }

    pub fn start_pomodoro(&mut self, now: Instant, durations: &Durations) {
        self.kind = Kind::Pomodoro;
        self.completed_focus = 0;
        self.sessions_before_long_break = durations.sessions_before_long_break.max(1);
        self.load_phase(Phase::Focus, durations.focus);
        self.run_from(now);
    }

    pub fn start_countdown(&mut self, now: Instant, seconds: u32) {
        self.kind = Kind::Countdown;
        self.load_phase(Phase::Countdown, seconds.max(1));
        self.run_from(now);
    }

    pub fn pause(&mut self, now: Instant) {
        if self.status == Status::Running {
            self.remaining_frozen = self.remaining(now);
            self.deadline = None;
            self.status = Status::Paused;
        }
    }

    /// Reprend après une pause, ou démarre la phase en attente.
    pub fn resume(&mut self, now: Instant) {
        if matches!(self.status, Status::Paused | Status::Ready) {
            self.run_from(now);
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        match (self.status, self.deadline) {
            (Status::Running, Some(deadline)) => deadline.saturating_duration_since(now),
            _ => self.remaining_frozen,
        }
    }

    fn next_phase(&mut self, finished: Phase) -> Option<Phase> {
        match finished {
            Phase::Focus => {
                self.completed_focus += 1;
                if self.completed_focus % self.sessions_before_long_break == 0 {
                    Some(Phase::LongBreak)
                } else {
                    Some(Phase::ShortBreak)
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Some(Phase::Focus),
            Phase::Countdown => None,
        }
    }

    fn advance(&mut self, now: Instant, durations: &Durations, auto_start: bool) -> Option<Finished> {
        let finished_phase = self.phase?;
        let next = self.next_phase(finished_phase);

        match next {
            Some(phase) => {
                self.sessions_before_long_break = durations.sessions_before_long_break.max(1);
                self.load_phase(phase, durations.for_phase(phase));
                if auto_start {
                    self.run_from(now);
                }
            }
            None => self.reset(),
        }

        Some(Finished { phase: finished_phase, next_phase: next })
    }

    /// À appeler régulièrement : termine la phase si l'échéance est passée.
    pub fn tick(&mut self, now: Instant, durations: &Durations, auto_start: bool) -> Option<Finished> {
        if self.status == Status::Running && self.deadline.is_some_and(|deadline| now >= deadline) {
            return self.advance(now, durations, auto_start);
        }
        None
    }

    /// Passe à la phase suivante du Pomodoro sans attendre la fin de la phase en cours.
    pub fn skip(&mut self, now: Instant, durations: &Durations, auto_start: bool) -> Option<Finished> {
        if self.kind != Kind::Pomodoro || self.status == Status::Idle {
            return None;
        }
        self.advance(now, durations, auto_start)
    }

    pub fn snapshot(&self, now: Instant) -> Snapshot {
        let remaining = self.remaining(now);
        // Arrondi vers le haut : un affichage à 0:00 ne doit apparaître qu'à l'échéance réelle.
        let remaining_seconds = remaining.as_secs() as u32 + u32::from(remaining.subsec_nanos() > 0);

        let cycle = self.sessions_before_long_break.max(1);
        let in_break = matches!(self.phase, Some(Phase::ShortBreak | Phase::LongBreak));
        let cycle_progress = if in_break && self.completed_focus > 0 && self.completed_focus % cycle == 0 {
            cycle
        } else {
            self.completed_focus % cycle
        };

        Snapshot {
            kind: self.kind,
            phase: self.phase,
            status: self.status,
            remaining_seconds,
            total_seconds: self.total.as_secs() as u32,
            completed_focus_sessions: self.completed_focus,
            sessions_before_long_break: cycle,
            cycle_progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DURATIONS: Durations = Durations {
        focus: 25 * 60,
        short_break: 5 * 60,
        long_break: 15 * 60,
        sessions_before_long_break: 4,
    };

    fn after(base: Instant, seconds: u64) -> Instant {
        base + Duration::from_secs(seconds)
    }

    #[test]
    fn countdown_counts_down_then_resets() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_countdown(t0, 60);

        assert_eq!(machine.snapshot(after(t0, 10)).remaining_seconds, 50);
        assert!(machine.tick(after(t0, 59), &DURATIONS, false).is_none());

        let finished = machine.tick(after(t0, 60), &DURATIONS, false).expect("doit se terminer");
        assert_eq!(finished, Finished { phase: Phase::Countdown, next_phase: None });
        assert_eq!(machine.snapshot(after(t0, 61)).status, Status::Idle);
    }

    #[test]
    fn pause_freezes_remaining_time_and_resume_continues() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_countdown(t0, 100);

        machine.pause(after(t0, 30));
        assert_eq!(machine.snapshot(after(t0, 500)).remaining_seconds, 70);
        assert_eq!(machine.status(), Status::Paused);

        machine.resume(after(t0, 500));
        assert_eq!(machine.snapshot(after(t0, 510)).remaining_seconds, 60);
    }

    #[test]
    fn pomodoro_alternates_focus_and_short_break() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_pomodoro(t0, &DURATIONS);
        assert_eq!(machine.snapshot(t0).phase, Some(Phase::Focus));

        let end_focus = after(t0, 25 * 60);
        let finished = machine.tick(end_focus, &DURATIONS, false).unwrap();
        assert_eq!(finished.next_phase, Some(Phase::ShortBreak));

        let snapshot = machine.snapshot(end_focus);
        assert_eq!(snapshot.status, Status::Ready);
        assert_eq!(snapshot.remaining_seconds, 5 * 60);
        assert_eq!(snapshot.completed_focus_sessions, 1);
    }

    #[test]
    fn long_break_comes_after_the_configured_number_of_sessions() {
        let mut now = Instant::now();
        let mut machine = Machine::new();
        machine.start_pomodoro(now, &DURATIONS);

        for session in 1..=4 {
            now = after(now, 25 * 60);
            let finished = machine.tick(now, &DURATIONS, true).unwrap();
            let expected = if session == 4 { Phase::LongBreak } else { Phase::ShortBreak };
            assert_eq!(finished.next_phase, Some(expected), "après la séance {session}");

            if session < 4 {
                now = after(now, 5 * 60);
                let back = machine.tick(now, &DURATIONS, true).unwrap();
                assert_eq!(back.next_phase, Some(Phase::Focus));
            }
        }
        assert_eq!(machine.snapshot(now).cycle_progress, 4);
    }

    #[test]
    fn auto_start_runs_the_next_phase_immediately() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_pomodoro(t0, &DURATIONS);

        let end = after(t0, 25 * 60);
        machine.tick(end, &DURATIONS, true).unwrap();
        assert_eq!(machine.status(), Status::Running);
        assert_eq!(machine.snapshot(after(end, 60)).remaining_seconds, 4 * 60);
    }

    #[test]
    fn manual_mode_waits_for_resume() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_pomodoro(t0, &DURATIONS);
        let end = after(t0, 25 * 60);
        machine.tick(end, &DURATIONS, false).unwrap();

        assert_eq!(machine.snapshot(after(end, 120)).remaining_seconds, 5 * 60);
        machine.resume(after(end, 120));
        assert_eq!(machine.snapshot(after(end, 180)).remaining_seconds, 4 * 60);
    }

    #[test]
    fn skip_moves_to_next_phase_only_for_pomodoro() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_countdown(t0, 60);
        assert!(machine.skip(t0, &DURATIONS, false).is_none());

        machine.reset();
        machine.start_pomodoro(t0, &DURATIONS);
        let finished = machine.skip(after(t0, 10), &DURATIONS, false).unwrap();
        assert_eq!(finished.phase, Phase::Focus);
        assert_eq!(machine.snapshot(after(t0, 10)).phase, Some(Phase::ShortBreak));
    }

    #[test]
    fn display_rounds_up_so_zero_only_shows_at_the_deadline() {
        let t0 = Instant::now();
        let mut machine = Machine::new();
        machine.start_countdown(t0, 10);
        let almost = t0 + Duration::from_millis(9_400);
        assert_eq!(machine.snapshot(almost).remaining_seconds, 1);
    }
}
