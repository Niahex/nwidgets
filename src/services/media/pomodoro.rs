use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use tokio::time::interval;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
pub struct PomodoroState {
    pub remaining_seconds: u32,
    pub is_running: bool,
    pub is_break: bool,
    pub work_duration: u32,
    pub break_duration: u32,
    pub completed_sessions: u32,
}

#[derive(Clone)]
pub struct PomodoroService {
    state: Arc<RwLock<PomodoroState>>,
}

impl PomodoroService {
    pub fn new() -> Self {
        let mut initial_state = PomodoroState::default();
        initial_state.work_duration = 25 * 60;
        initial_state.break_duration = 5 * 60;
        initial_state.remaining_seconds = 25 * 60;

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
                        if s.is_break {
                            s.is_break = false;
                            s.remaining_seconds = s.work_duration;
                        } else {
                            s.is_break = true;
                            s.remaining_seconds = s.break_duration;
                            s.completed_sessions += 1;
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
    }

    pub fn reset(&self) {
        let mut state = self.state.write();
        state.is_running = false;
        state.is_break = false;
        state.remaining_seconds = state.work_duration;
    }

    pub fn get_state(&self) -> PomodoroState {
        self.state.read().clone()
    }
}
