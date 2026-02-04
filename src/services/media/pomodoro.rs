use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use tokio::time::interval;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, PartialEq)]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Default for PomodoroPhase {
    fn default() -> Self {
        Self::Work
    }
}

#[derive(Clone, Debug, Default)]
pub struct PomodoroState {
    pub remaining_seconds: u32,
    pub is_running: bool,
    pub phase: PomodoroPhase,
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
    pub completed_sessions: u32,
    pub pomodoros_until_long_break: u32,
    pub has_started: bool,
}

#[derive(Clone)]
pub struct PomodoroService {
    state: Arc<RwLock<PomodoroState>>,
}

impl PomodoroService {
    pub fn new() -> Self {
        let mut initial_state = PomodoroState::default();
        initial_state.work_duration = 25 * 60;
        initial_state.short_break_duration = 5 * 60;
        initial_state.long_break_duration = 15 * 60;
        initial_state.remaining_seconds = 25 * 60;
        initial_state.phase = PomodoroPhase::Work;
        initial_state.pomodoros_until_long_break = 4;

        let service = Self {
            state: Arc::new(RwLock::new(initial_state)),
        };

        service.start_timer();
        service
    }

    fn start_timer(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));

            loop {
                ticker.tick().await;

                let mut s = state.write();
                if s.is_running && s.remaining_seconds > 0 {
                    s.remaining_seconds -= 1;

                    if s.remaining_seconds == 0 {
                        match s.phase {
                            PomodoroPhase::Work => {
                                s.completed_sessions += 1;
                                
                                let is_long_break = s.completed_sessions % s.pomodoros_until_long_break == 0;
                                
                                if is_long_break {
                                    s.phase = PomodoroPhase::LongBreak;
                                    s.remaining_seconds = s.long_break_duration;
                                } else {
                                    s.phase = PomodoroPhase::ShortBreak;
                                    s.remaining_seconds = s.short_break_duration;
                                }
                            }
                            PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => {
                                s.phase = PomodoroPhase::Work;
                                s.remaining_seconds = s.work_duration;
                            }
                        }
                        s.is_running = false;
                    }
                }
            }
        });
    }

    pub fn toggle(&self) {
        let mut state = self.state.write();
        state.is_running = !state.is_running;
        if state.is_running {
            state.has_started = true;
        }
    }

    pub fn reset(&self) {
        let mut state = self.state.write();
        state.is_running = false;
        state.phase = PomodoroPhase::Work;
        state.remaining_seconds = state.work_duration;
        state.has_started = false;
    }

    pub fn get_state(&self) -> PomodoroState {
        self.state.read().clone()
    }
}
