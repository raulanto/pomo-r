use std::time::{Duration, Instant};

use super::phase::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseFinished {
    pub finished_phase: Phase,
    pub next_phase: Phase,
    pub completed_pomodoros: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    phase: Phase,
    status: Status,
    remaining: Duration,
    completed_pomodoros: u32,
    work_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
    long_break_every: u32,
    last_tick: Option<Instant>,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
    }
}

impl Timer {
    pub fn new(
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        long_break_every: u32,
    ) -> Self {
        let work = work_duration.max(Duration::from_secs(60));
        let short = short_break_duration.max(Duration::from_secs(60));
        let long = long_break_duration.max(Duration::from_secs(60));
        let cycles = long_break_every.max(1);

        Self {
            phase: Phase::Work,
            status: Status::Idle,
            remaining: work,
            completed_pomodoros: 0,
            work_duration: work,
            short_break_duration: short,
            long_break_duration: long,
            long_break_every: cycles,
            last_tick: None,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn completed_pomodoros(&self) -> u32 {
        self.completed_pomodoros
    }

    pub fn long_break_every(&self) -> u32 {
        self.long_break_every
    }

    pub fn work_duration(&self) -> Duration {
        self.work_duration
    }

    pub fn short_break_duration(&self) -> Duration {
        self.short_break_duration
    }

    pub fn long_break_duration(&self) -> Duration {
        self.long_break_duration
    }

    pub fn duration_for_phase(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Work => self.work_duration,
            Phase::ShortBreak => self.short_break_duration,
            Phase::LongBreak => self.long_break_duration,
        }
    }

    pub fn total_duration(&self) -> Duration {
        self.duration_for_phase(self.phase)
    }

    pub fn start(&mut self) {
        if self.status != Status::Running {
            self.status = Status::Running;
            self.last_tick = None;
        }
    }

    pub fn pause(&mut self) {
        if self.status == Status::Running {
            self.status = Status::Paused;
            self.last_tick = None;
        }
    }

    pub fn toggle(&mut self) {
        match self.status {
            Status::Running => self.pause(),
            Status::Paused | Status::Idle => self.start(),
        }
    }

    pub fn reset(&mut self) {
        self.remaining = self.duration_for_phase(self.phase);
        self.status = Status::Idle;
        self.last_tick = None;
    }

    pub fn select_phase(&mut self, phase: Phase) {
        self.phase = phase;
        self.remaining = self.duration_for_phase(phase);
        self.status = Status::Idle;
        self.last_tick = None;
    }

    pub fn skip(&mut self) {
        let next_phase = match self.phase {
            Phase::Work => Phase::ShortBreak,
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };
        self.select_phase(next_phase);
    }

    pub fn adjust(&mut self, delta_minutes: i32) {
        let one_min = Duration::from_secs(60);
        if delta_minutes > 0 {
            let add = Duration::from_secs((delta_minutes as u64) * 60);
            self.remaining = self.remaining.saturating_add(add);
        } else if delta_minutes < 0 {
            let sub = Duration::from_secs((-delta_minutes as u64) * 60);
            self.remaining = self.remaining.saturating_sub(sub).max(one_min);
        }
    }

    pub fn tick(&mut self, now: Instant) -> Option<PhaseFinished> {
        if self.status != Status::Running {
            self.last_tick = None;
            return None;
        }

        let prev = match self.last_tick {
            Some(prev) => prev,
            None => {
                self.last_tick = Some(now);
                return None;
            }
        };

        let elapsed = now.saturating_duration_since(prev);
        self.last_tick = Some(now);

        if elapsed.is_zero() {
            return None;
        }

        if elapsed >= self.remaining {
            self.remaining = Duration::ZERO;
            let finished_phase = self.phase;
            let next_phase = match finished_phase {
                Phase::Work => {
                    self.completed_pomodoros = self.completed_pomodoros.saturating_add(1);
                    if self
                        .completed_pomodoros
                        .is_multiple_of(self.long_break_every)
                    {
                        Phase::LongBreak
                    } else {
                        Phase::ShortBreak
                    }
                }
                Phase::ShortBreak | Phase::LongBreak => Phase::Work,
            };

            self.phase = next_phase;
            self.remaining = self.duration_for_phase(next_phase);
            self.status = Status::Idle;
            self.last_tick = None;

            Some(PhaseFinished {
                finished_phase,
                next_phase,
                completed_pomodoros: self.completed_pomodoros,
            })
        } else {
            self.remaining -= elapsed;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_timer() -> Timer {
        Timer::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
    }

    #[test]
    fn test_work_to_short_break_transition() {
        let mut timer = create_test_timer();
        timer.start();

        let t0 = Instant::now();
        assert_eq!(timer.tick(t0), None);

        let t1 = t0 + Duration::from_secs(25 * 60);
        let event = timer.tick(t1);

        assert_eq!(
            event,
            Some(PhaseFinished {
                finished_phase: Phase::Work,
                next_phase: Phase::ShortBreak,
                completed_pomodoros: 1,
            })
        );
        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.completed_pomodoros(), 1);
        assert_eq!(timer.remaining(), Duration::from_secs(5 * 60));
        assert_eq!(timer.status(), Status::Idle);
    }

    #[test]
    fn test_fourth_pomodoro_to_long_break_transition() {
        let mut timer = create_test_timer();

        let mut t = Instant::now();
        for expected_pomodoro in 1..=4 {
            timer.start();
            assert_eq!(timer.tick(t), None);
            t += timer.remaining();
            let event = timer.tick(t);

            if expected_pomodoro == 4 {
                assert_eq!(
                    event,
                    Some(PhaseFinished {
                        finished_phase: Phase::Work,
                        next_phase: Phase::LongBreak,
                        completed_pomodoros: 4,
                    })
                );
                assert_eq!(timer.phase(), Phase::LongBreak);
                assert_eq!(timer.remaining(), Duration::from_secs(15 * 60));
            } else {
                assert_eq!(
                    event,
                    Some(PhaseFinished {
                        finished_phase: Phase::Work,
                        next_phase: Phase::ShortBreak,
                        completed_pomodoros: expected_pomodoro,
                    })
                );
                // Advance break to get back to Work
                timer.start();
                assert_eq!(timer.tick(t), None);
                t += timer.remaining();
                let break_event = timer.tick(t);
                assert_eq!(
                    break_event,
                    Some(PhaseFinished {
                        finished_phase: Phase::ShortBreak,
                        next_phase: Phase::Work,
                        completed_pomodoros: expected_pomodoro,
                    })
                );
            }
        }
    }

    #[test]
    fn test_break_to_work_transition() {
        let mut timer = create_test_timer();
        timer.select_phase(Phase::ShortBreak);
        timer.start();

        let t0 = Instant::now();
        assert_eq!(timer.tick(t0), None);
        let t1 = t0 + Duration::from_secs(5 * 60);
        let event = timer.tick(t1);

        assert_eq!(
            event,
            Some(PhaseFinished {
                finished_phase: Phase::ShortBreak,
                next_phase: Phase::Work,
                completed_pomodoros: 0,
            })
        );
        assert_eq!(timer.phase(), Phase::Work);
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));
    }

    #[test]
    fn test_skip() {
        let mut timer = create_test_timer();
        assert_eq!(timer.phase(), Phase::Work);

        timer.skip();
        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.remaining(), Duration::from_secs(5 * 60));
        assert_eq!(timer.status(), Status::Idle);

        timer.skip();
        assert_eq!(timer.phase(), Phase::Work);
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));

        timer.select_phase(Phase::LongBreak);
        assert_eq!(timer.phase(), Phase::LongBreak);

        timer.skip();
        assert_eq!(timer.phase(), Phase::Work);
    }

    #[test]
    fn test_reset() {
        let mut timer = create_test_timer();
        timer.start();

        let t0 = Instant::now();
        timer.tick(t0);
        timer.tick(t0 + Duration::from_secs(5 * 60));
        assert_eq!(timer.remaining(), Duration::from_secs(20 * 60));

        timer.reset();
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));
        assert_eq!(timer.status(), Status::Idle);
    }

    #[test]
    fn test_adjust_with_minimum() {
        let mut timer = create_test_timer();
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));

        timer.adjust(5);
        assert_eq!(timer.remaining(), Duration::from_secs(30 * 60));

        timer.adjust(-10);
        assert_eq!(timer.remaining(), Duration::from_secs(20 * 60));

        // Attempting to adjust well below zero should clamp to minimum 1 minute (60s)
        timer.adjust(-50);
        assert_eq!(timer.remaining(), Duration::from_secs(60));

        // Adjusting negative again maintains minimum 1 minute
        timer.adjust(-5);
        assert_eq!(timer.remaining(), Duration::from_secs(60));
    }

    #[test]
    fn test_tick_when_paused_does_not_advance() {
        let mut timer = create_test_timer();
        timer.start();

        let t0 = Instant::now();
        timer.tick(t0);
        timer.tick(t0 + Duration::from_secs(60));
        assert_eq!(timer.remaining(), Duration::from_secs(24 * 60));

        timer.pause();
        assert_eq!(timer.status(), Status::Paused);

        // Tick while paused
        let event = timer.tick(t0 + Duration::from_secs(120));
        assert_eq!(event, None);
        assert_eq!(timer.remaining(), Duration::from_secs(24 * 60));
    }

    #[test]
    fn test_pause_and_resume_does_not_count_paused_time() {
        let mut timer = create_test_timer();
        timer.start();

        let t0 = Instant::now();
        // First tick initializes last_tick
        timer.tick(t0);
        // 1 second elapses
        timer.tick(t0 + Duration::from_secs(1));
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60 - 1));

        // Pause for 10 seconds
        timer.pause();
        timer.tick(t0 + Duration::from_secs(11));
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60 - 1));

        // Resume at t0 + 11s
        timer.start();
        // First tick after resume re-initializes last_tick, elapsed = 0
        timer.tick(t0 + Duration::from_secs(11));
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60 - 1));

        // Advance 2 seconds after resumption
        timer.tick(t0 + Duration::from_secs(13));
        // Only 1s before pause + 2s after pause = 3s total elapsed
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60 - 3));
    }
}
