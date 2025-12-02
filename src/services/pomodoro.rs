use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PomodoroState {
    Idle,
    Work,
    WorkPaused,
    ShortBreak,
    ShortBreakPaused,
    LongBreak,
    LongBreakPaused,
}

pub struct PomodoroService {
    state: PomodoroState,
    start_time: Option<Instant>,
    paused_time: Option<Instant>,
    elapsed_before_pause: Duration,
    work_duration: Duration, // 25 minutes
    short_break: Duration,   // 5 minutes
    long_break: Duration,    // 15 minutes
    sessions_completed: u8,  // Pour compter les sessions avant la longue pause
}

impl PomodoroService {
    pub fn new() -> Self {
        Self {
            state: PomodoroState::Idle,
            start_time: None,
            paused_time: None,
            elapsed_before_pause: Duration::ZERO,
            work_duration: Duration::from_secs(25 * 60),
            short_break: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(15 * 60),
            sessions_completed: 0,
        }
    }

    pub fn pause(&mut self) {
        match self.state {
            PomodoroState::Work => {
                if let Some(start) = self.start_time {
                    self.elapsed_before_pause = start.elapsed();
                    self.paused_time = Some(Instant::now());
                    self.state = PomodoroState::WorkPaused;
                    println!("[POMODORO] ⏸️  Work paused at {}", self.format_time());
                }
            }
            PomodoroState::ShortBreak => {
                if let Some(start) = self.start_time {
                    self.elapsed_before_pause = start.elapsed();
                    self.paused_time = Some(Instant::now());
                    self.state = PomodoroState::ShortBreakPaused;
                    println!("[POMODORO] ⏸️  Short break paused");
                }
            }
            PomodoroState::LongBreak => {
                if let Some(start) = self.start_time {
                    self.elapsed_before_pause = start.elapsed();
                    self.paused_time = Some(Instant::now());
                    self.state = PomodoroState::LongBreakPaused;
                    println!("[POMODORO] ⏸️  Long break paused");
                }
            }
            _ => {}
        }
    }

    pub fn resume(&mut self) {
        match self.state {
            PomodoroState::WorkPaused => {
                self.start_time = Some(Instant::now() - self.elapsed_before_pause);
                self.paused_time = None;
                self.state = PomodoroState::Work;
                println!("[POMODORO] ▶️  Work resumed");
            }
            PomodoroState::ShortBreakPaused => {
                self.start_time = Some(Instant::now() - self.elapsed_before_pause);
                self.paused_time = None;
                self.state = PomodoroState::ShortBreak;
                println!("[POMODORO] ▶️  Short break resumed");
            }
            PomodoroState::LongBreakPaused => {
                self.start_time = Some(Instant::now() - self.elapsed_before_pause);
                self.paused_time = None;
                self.state = PomodoroState::LongBreak;
                println!("[POMODORO] ▶️  Long break resumed");
            }
            _ => {}
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(
            self.state,
            PomodoroState::WorkPaused
                | PomodoroState::ShortBreakPaused
                | PomodoroState::LongBreakPaused
        )
    }

    pub fn start_work(&mut self) {
        self.state = PomodoroState::Work;
        self.start_time = Some(Instant::now());
        println!("[POMODORO] 🍅 Work session started");
    }

    pub fn start_short_break(&mut self) {
        self.state = PomodoroState::ShortBreak;
        self.start_time = Some(Instant::now());
        println!("[POMODORO] ☕ Short break started");
    }

    pub fn start_long_break(&mut self) {
        self.state = PomodoroState::LongBreak;
        self.start_time = Some(Instant::now());
        println!("[POMODORO] 🌴 Long break started");
    }

    pub fn stop(&mut self) {
        self.state = PomodoroState::Idle;
        self.start_time = None;
        println!("[POMODORO] ⏹️  Stopped");
    }

    pub fn reset(&mut self) {
        self.state = PomodoroState::Idle;
        self.start_time = None;
        self.paused_time = None;
        self.elapsed_before_pause = Duration::ZERO;
        self.sessions_completed = 0;
        println!("[POMODORO] 🔄 Reset");
    }

    #[allow(dead_code)]
    pub fn toggle(&mut self) {
        match self.state {
            PomodoroState::Idle => self.start_work(),
            _ => self.stop(),
        }
    }

    pub fn get_state(&self) -> PomodoroState {
        self.state
    }

    pub fn get_remaining_seconds(&self) -> u32 {
        let elapsed = if self.is_paused() {
            // When paused, use the elapsed time captured at pause
            self.elapsed_before_pause
        } else if let Some(start) = self.start_time {
            // When running, calculate current elapsed time
            start.elapsed()
        } else {
            // No timer running
            return 0;
        };

        let total_duration = match self.state {
            PomodoroState::Work | PomodoroState::WorkPaused => self.work_duration,
            PomodoroState::ShortBreak | PomodoroState::ShortBreakPaused => self.short_break,
            PomodoroState::LongBreak | PomodoroState::LongBreakPaused => self.long_break,
            PomodoroState::Idle => return 0,
        };

        if elapsed >= total_duration {
            return 0;
        }

        (total_duration - elapsed).as_secs() as u32
    }

    pub fn is_finished(&self) -> bool {
        // Don't auto-transition when paused
        if self.is_paused() {
            return false;
        }

        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            let total_duration = match self.state {
                PomodoroState::Work => self.work_duration,
                PomodoroState::ShortBreak => self.short_break,
                PomodoroState::LongBreak => self.long_break,
                _ => return false,
            };

            elapsed >= total_duration
        } else {
            false
        }
    }

    pub fn auto_transition(&mut self) {
        if self.is_finished() {
            match self.state {
                PomodoroState::Work => {
                    self.sessions_completed += 1;
                    println!(
                        "[POMODORO] ✅ Work session completed ({}/4)",
                        self.sessions_completed
                    );

                    if self.sessions_completed >= 4 {
                        self.sessions_completed = 0;
                        self.start_long_break();
                    } else {
                        self.start_short_break();
                    }
                }
                PomodoroState::ShortBreak => {
                    println!("[POMODORO] ✅ Short break completed");
                    self.start_work();
                }
                PomodoroState::LongBreak => {
                    println!("[POMODORO] ✅ Long break completed");
                    self.stop();
                }
                PomodoroState::Idle
                | PomodoroState::WorkPaused
                | PomodoroState::ShortBreakPaused
                | PomodoroState::LongBreakPaused => {}
            }
        }
    }

    pub fn format_time(&self) -> String {
        let seconds = self.get_remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }
}
