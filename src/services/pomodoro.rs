use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum PomodoroPhase {
    Work,
    #[allow(dead_code)]
    ShortBreak,
    #[allow(dead_code)]
    LongBreak,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PomodoroStatus {
    Idle,
    Running {
        phase: PomodoroPhase,
        remaining_secs: u32,
    },
    Paused {
        phase: PomodoroPhase,
        remaining_secs: u32,
    },
}

#[derive(Clone)]
pub struct PomodoroStateChanged;

pub struct PomodoroService {
    status: Arc<RwLock<PomodoroStatus>>,
    work_duration: u32,
    #[allow(dead_code)]
    short_break_duration: u32,
    #[allow(dead_code)]
    long_break_duration: u32,
    #[allow(dead_code)]
    pomodoros_until_long_break: u32,
    pomodoro_count: Arc<RwLock<u32>>,
    start_time: Arc<RwLock<Option<Instant>>>,
}

impl EventEmitter<PomodoroStateChanged> for PomodoroService {}

impl PomodoroService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let status = Arc::new(RwLock::new(PomodoroStatus::Idle));
        let pomodoro_count = Arc::new(RwLock::new(0));
        let start_time = Arc::new(RwLock::new(None));

        let status_clone = Arc::clone(&status);
        let start_time_clone = Arc::clone(&start_time);

        // Update timer every second
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                Self::monitor_timer(this, status_clone, start_time_clone, &mut cx).await
            }
        })
        .detach();

        Self {
            status,
            work_duration: 25 * 60,       // 25 minutes
            short_break_duration: 5 * 60, // 5 minutes
            long_break_duration: 15 * 60, // 15 minutes
            pomodoros_until_long_break: 4,
            pomodoro_count,
            start_time,
        }
    }

    pub fn status(&self) -> PomodoroStatus {
        self.status.read().clone()
    }

    pub fn start_work(&self, cx: &mut Context<Self>) {
        let duration = self.work_duration;
        *self.status.write() = PomodoroStatus::Running {
            phase: PomodoroPhase::Work,
            remaining_secs: duration,
        };
        *self.start_time.write() = Some(Instant::now());
        cx.emit(PomodoroStateChanged);
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn start_break(&self, cx: &mut Context<Self>) {
        let count = *self.pomodoro_count.read();
        let is_long_break = count % self.pomodoros_until_long_break == 0 && count > 0;

        let (phase, duration) = if is_long_break {
            (PomodoroPhase::LongBreak, self.long_break_duration)
        } else {
            (PomodoroPhase::ShortBreak, self.short_break_duration)
        };

        *self.status.write() = PomodoroStatus::Running {
            phase,
            remaining_secs: duration,
        };
        *self.start_time.write() = Some(Instant::now());
        cx.emit(PomodoroStateChanged);
        cx.notify();
    }

    pub fn pause(&self, cx: &mut Context<Self>) {
        let current_status = self.status.read().clone();
        if let PomodoroStatus::Running {
            phase,
            remaining_secs,
        } = current_status
        {
            *self.status.write() = PomodoroStatus::Paused {
                phase,
                remaining_secs,
            };
            *self.start_time.write() = None;
            cx.emit(PomodoroStateChanged);
            cx.notify();
        }
    }

    pub fn resume(&self, cx: &mut Context<Self>) {
        let current_status = self.status.read().clone();
        if let PomodoroStatus::Paused {
            phase,
            remaining_secs,
        } = current_status
        {
            *self.status.write() = PomodoroStatus::Running {
                phase,
                remaining_secs,
            };
            *self.start_time.write() = Some(Instant::now());
            cx.emit(PomodoroStateChanged);
            cx.notify();
        }
    }

    pub fn stop(&self, cx: &mut Context<Self>) {
        *self.status.write() = PomodoroStatus::Idle;
        *self.start_time.write() = None;
        cx.emit(PomodoroStateChanged);
        cx.notify();
    }

    async fn monitor_timer(
        this: WeakEntity<Self>,
        status: Arc<RwLock<PomodoroStatus>>,
        _start_time: Arc<RwLock<Option<Instant>>>,
        cx: &mut AsyncApp,
    ) {
        loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;

            let should_update = {
                let current_status = status.read().clone();
                matches!(current_status, PomodoroStatus::Running { .. })
            };

            if should_update {
                let _ = this.update(cx, |service, cx| {
                    let mut current_status = service.status.write();

                    if let PomodoroStatus::Running {
                        phase,
                        remaining_secs,
                    } = &*current_status
                    {
                        if *remaining_secs > 0 {
                            *current_status = PomodoroStatus::Running {
                                phase: phase.clone(),
                                remaining_secs: remaining_secs - 1,
                            };

                            cx.emit(PomodoroStateChanged);
                            cx.notify();
                        } else {
                            // Timer finished
                            if matches!(phase, PomodoroPhase::Work) {
                                *service.pomodoro_count.write() += 1;
                            }
                            *current_status = PomodoroStatus::Idle;
                            *service.start_time.write() = None;

                            cx.emit(PomodoroStateChanged);
                            cx.notify();
                        }
                    }
                });
            }
        }
    }
}

// Global accessor
struct GlobalPomodoroService(Entity<PomodoroService>);
impl Global for GlobalPomodoroService {}

impl PomodoroService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalPomodoroService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalPomodoroService(service.clone()));
        service
    }
}