use crate::services::clipboard::ClipboardService;
use crate::services::osd::{OsdEvent, OsdEventService};
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SizedSample};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use transcribe_rs::{
    engines::parakeet::{ParakeetEngine, ParakeetInferenceParams, ParakeetModelParams, TimestampGranularity},
    TranscriptionEngine,
};

const WHISPER_SAMPLE_RATE: u32 = 16000;
const SILENCE_THRESHOLD: f32 = 0.01;
const SILENCE_DURATION_MS: u128 = 2000;

#[derive(Debug, Clone, PartialEq)]
pub enum SttState {
    Idle,
    Recording,
    Processing,
    Error(String),
}

#[allow(dead_code)]
pub enum SttEvent {
    StateChanged(SttState),
    TranscriptionComplete(String),
    AutoStopped,
}

enum SttCommand {
    Start,
    Stop,
    #[allow(dead_code)]
    Shutdown,
}

struct AudioRecorder {
    cmd_tx: mpsc::Sender<SttCommand>,
}

impl AudioRecorder {
    fn new(event_tx: mpsc::Sender<AudioEvent>) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(anyhow!("No input device found"))?;

        let (sample_tx, sample_rx) = mpsc::channel::<Vec<f32>>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<SttCommand>();

        thread::spawn(move || {
            if let Err(e) = Self::run_audio_thread(device, sample_tx, sample_rx, cmd_rx, event_tx) {
                eprintln!("Audio thread error: {e}");
            }
        });

        Ok(Self { cmd_tx })
    }

    fn start(&self) -> Result<()> {
        self.cmd_tx
            .send(SttCommand::Start)
            .map_err(|e| anyhow!("Failed to send Start: {e}"))
    }

    fn stop(&self) -> Result<()> {
        self.cmd_tx
            .send(SttCommand::Stop)
            .map_err(|e| anyhow!("Failed to send Stop: {e}"))
    }

    fn run_audio_thread(
        device: Device,
        sample_tx: mpsc::Sender<Vec<f32>>,
        sample_rx: mpsc::Receiver<Vec<f32>>,
        cmd_rx: mpsc::Receiver<SttCommand>,
        event_tx: mpsc::Sender<AudioEvent>,
    ) -> Result<()> {
        let config = Self::get_preferred_config(&device)?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                Self::build_stream::<f32>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::I16 => {
                Self::build_stream::<i16>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::U16 => {
                Self::build_stream::<u16>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::I8 => {
                Self::build_stream::<i8>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::U8 => {
                Self::build_stream::<u8>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::I32 => {
                Self::build_stream::<i32>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::U32 => {
                Self::build_stream::<u32>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::F64 => {
                Self::build_stream::<f64>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::I64 => {
                Self::build_stream::<i64>(&device, &config.into(), sample_tx.clone(), channels)
            }
            cpal::SampleFormat::U64 => {
                Self::build_stream::<u64>(&device, &config.into(), sample_tx.clone(), channels)
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                ))
            }
        }?;

        stream.play()?;

        let mut buffer = Vec::with_capacity(16000 * 600);
        let mut recording = false;
        let mut last_speech_time = Instant::now();

        loop {
            // Check for commands
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    SttCommand::Start => {
                        buffer.clear();
                        recording = true;
                        last_speech_time = Instant::now();
                    }
                    SttCommand::Stop => {
                        recording = false;
                        let mut final_samples = Self::process_buffer(&buffer, sample_rate);
                        Self::trim_silence(&mut final_samples, SILENCE_THRESHOLD);
                        let _ = event_tx.send(AudioEvent::ManualStopped(final_samples));
                        buffer.clear();
                    }
                    SttCommand::Shutdown => break,
                }
            }

            // Process audio
            match sample_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(chunk) => {
                    if recording {
                        let max_amplitude = chunk.iter().fold(0.0f32, |max, &x| max.max(x.abs()));

                        if max_amplitude > SILENCE_THRESHOLD {
                            last_speech_time = Instant::now();
                        } else if last_speech_time.elapsed().as_millis() > SILENCE_DURATION_MS
                            && !buffer.is_empty()
                        {
                            recording = false;
                            let mut final_samples = Self::process_buffer(&buffer, sample_rate);
                            Self::trim_silence(&mut final_samples, SILENCE_THRESHOLD);
                            let _ = event_tx.send(AudioEvent::AutoStopped(final_samples));
                            buffer.clear();
                        }

                        if recording {
                            buffer.extend_from_slice(&chunk);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(())
    }

    fn build_stream<T>(
        device: &Device,
        config: &cpal::StreamConfig,
        tx: mpsc::Sender<Vec<f32>>,
        channels: usize,
    ) -> Result<cpal::Stream>
    where
        T: SizedSample + Sample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                let mut output = Vec::with_capacity(data.len() / channels);
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|s| s.to_sample::<f32>()).sum();
                    output.push(sum / channels as f32);
                }
                let _ = tx.send(output);
            },
            |err| eprintln!("Stream error: {err}"),
            None,
        )?;
        Ok(stream)
    }

    fn get_preferred_config(device: &Device) -> Result<cpal::SupportedStreamConfig> {
        let configs = device.supported_input_configs()?;
        for config in configs {
            if config.min_sample_rate().0 <= WHISPER_SAMPLE_RATE
                && config.max_sample_rate().0 >= WHISPER_SAMPLE_RATE
            {
                return Ok(config.with_sample_rate(cpal::SampleRate(WHISPER_SAMPLE_RATE)));
            }
        }
        Ok(device.default_input_config()?)
    }

    fn process_buffer(buffer: &[f32], sample_rate: u32) -> Vec<f32> {
        if sample_rate != WHISPER_SAMPLE_RATE {
            Self::resample_simple(buffer, sample_rate, WHISPER_SAMPLE_RATE)
        } else {
            buffer.to_vec()
        }
    }

    fn resample_simple(input: &[f32], in_rate: u32, out_rate: u32) -> Vec<f32> {
        let ratio = in_rate as f32 / out_rate as f32;
        let out_len = (input.len() as f32 / ratio) as usize;
        let mut output = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let index = i as f32 * ratio;
            let idx_floor = index.floor() as usize;
            let idx_ceil = (idx_floor + 1).min(input.len() - 1);
            let t = index - idx_floor as f32;
            let sample = input[idx_floor] * (1.0 - t) + input[idx_ceil] * t;
            output.push(sample);
        }
        output
    }

    fn trim_silence(samples: &mut Vec<f32>, threshold: f32) {
        if samples.is_empty() {
            return;
        }
        let start = samples
            .iter()
            .position(|&x| x.abs() > threshold)
            .unwrap_or(0);
        let end = samples
            .iter()
            .rposition(|&x| x.abs() > threshold)
            .unwrap_or(samples.len() - 1);
        if start >= end {
            samples.clear();
        } else {
            let padding = 3200;
            let start_pad = start.saturating_sub(padding);
            let end_pad = (end + padding).min(samples.len());
            *samples = samples[start_pad..end_pad].to_vec();
        }
    }
}

enum AudioEvent {
    AutoStopped(Vec<f32>),
    ManualStopped(Vec<f32>),
}

struct TranscriptionManager {
    engine: Arc<Mutex<Option<ParakeetEngine>>>,
    model_path: PathBuf,
}

impl TranscriptionManager {
    fn new() -> Self {
        let model_path = PathBuf::from("/home/nia/.local/share/ai")
            .join("parakeet-tdt-0.6b-v3-int8");

        Self {
            engine: Arc::new(Mutex::new(None)),
            model_path,
        }
    }

    fn ensure_model_exists(&self) -> Result<()> {
        if self.model_path.exists() {
            return Ok(());
        }

        // Create parent directory if needed
        if let Some(parent) = self.model_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("Downloading Parakeet V3 int8 model to {:?}", self.model_path);
        println!("This is a quantized model (~473MB), please wait...");
        let url = "https://blob.handy.computer/parakeet-v3-int8.tar.gz";

        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            let resp = reqwest::get(url).await?.bytes().await?;

            // Save tar.gz file
            let tar_gz_path = self.model_path.with_extension("tar.gz");
            std::fs::write(&tar_gz_path, resp)?;

            // Extract tar.gz
            let tar_gz = std::fs::File::open(&tar_gz_path)?;
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);

            if let Some(parent) = self.model_path.parent() {
                archive.unpack(parent)?;
            }

            // Remove tar.gz
            std::fs::remove_file(&tar_gz_path)?;

            Ok::<(), anyhow::Error>(())
        })?;

        println!("Model downloaded and extracted.");
        Ok(())
    }

    fn load_model(&self) -> Result<()> {
        self.ensure_model_exists()?;

        // Configure ONNX Runtime for i9-9900K optimization
        std::env::set_var("ORT_NUM_THREADS", "16"); // Use all 16 threads
        std::env::set_var("ORT_INTRA_OP_NUM_THREADS", "16");
        std::env::set_var("ORT_INTER_OP_NUM_THREADS", "1");
        std::env::set_var("ORT_OPTIMIZATION_LEVEL", "3"); // Max optimization

        // Create Parakeet engine
        let mut engine = ParakeetEngine::new();

        // Load with int8 quantization for speed
        engine
            .load_model_with_params(&self.model_path, ParakeetModelParams::int8())
            .map_err(|e| anyhow!("Failed to load Parakeet model: {e}"))?;

        let mut guard = self.engine.lock().unwrap();
        *guard = Some(engine);

        println!("Parakeet V3 int8 loaded (i9-9900K: 16 threads, AVX2/FMA enabled).");
        Ok(())
    }

    fn transcribe(&self, audio_data: &[f32]) -> Result<String> {
        let mut guard = self.engine.lock().unwrap();
        let engine = guard.as_mut().ok_or(anyhow!("Engine not loaded"))?;

        // Configure params for segment-level timestamps
        let params = ParakeetInferenceParams {
            timestamp_granularity: TimestampGranularity::Segment,
        };

        // Run transcription
        let result = engine
            .transcribe_samples(audio_data.to_vec(), Some(params))
            .map_err(|e| anyhow!("Parakeet transcription failed: {e}"))?;

        Ok(result.text)
    }
}

pub struct SttService {
    state: Arc<Mutex<SttState>>,
    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    transcriber: Arc<Mutex<Option<TranscriptionManager>>>,
}

impl SttService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SttState::Idle)),
            recorder: Arc::new(Mutex::new(None)),
            transcriber: Arc::new(Mutex::new(None)),
        }
    }

    pub fn initialize(&self) -> Result<()> {
        // Initialize transcription manager in background
        let transcriber = self.transcriber.clone();
        thread::spawn(move || {
            let manager = TranscriptionManager::new();
            if let Err(e) = manager.load_model() {
                eprintln!("Failed to load Whisper model: {e}");
                return;
            }
            *transcriber.lock().unwrap() = Some(manager);
            println!("STT Service initialized");
        });

        Ok(())
    }

    #[allow(dead_code)]
    pub fn subscribe<F>(callback: F)
    where
        F: Fn(SttEvent) + Send + 'static,
    {
        let (_tx, rx) = mpsc::channel();

        thread::spawn(move || {
            crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
        });

        // Keep tx alive somewhere or handle differently
        // This is a simplified version - you'd need to store tx properly
    }

    pub fn toggle(&self) -> Result<()> {
        let current_state = self.state.lock().unwrap().clone();

        match current_state {
            SttState::Idle => self.start_recording(),
            SttState::Recording => self.stop_recording(),
            _ => Ok(()), // Ignore toggle pendant le processing
        }
    }

    fn start_recording(&self) -> Result<()> {
        // Check if transcriber is ready
        if self.transcriber.lock().unwrap().is_none() {
            OsdEventService::send_event(OsdEvent::SttError("STT not initialized".to_string()));
            return Err(anyhow!("STT not initialized yet"));
        }

        let (event_tx, event_rx) = mpsc::channel();

        // Create recorder
        let recorder = match AudioRecorder::new(event_tx) {
            Ok(r) => r,
            Err(e) => {
                let error_msg = format!("Audio error: {e}");
                OsdEventService::send_event(OsdEvent::SttError(error_msg.clone()));
                return Err(anyhow!(error_msg));
            }
        };

        if let Err(e) = recorder.start() {
            let error_msg = format!("Start error: {e}");
            OsdEventService::send_event(OsdEvent::SttError(error_msg.clone()));
            return Err(anyhow!(error_msg));
        }

        *self.recorder.lock().unwrap() = Some(recorder);
        *self.state.lock().unwrap() = SttState::Recording;

        // Send OSD event
        OsdEventService::send_event(OsdEvent::SttRecording);

        // Handle audio events
        let state = self.state.clone();
        let transcriber = self.transcriber.clone();
        let recorder = self.recorder.clone();
        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                match event {
                    AudioEvent::AutoStopped(samples) | AudioEvent::ManualStopped(samples) => {
                        *state.lock().unwrap() = SttState::Processing;
                        OsdEventService::send_event(OsdEvent::SttProcessing);

                        // Clear recorder
                        *recorder.lock().unwrap() = None;

                        if let Some(manager) = transcriber.lock().unwrap().as_ref() {
                            match manager.transcribe(&samples) {
                                Ok(text) => {
                                    let trimmed = text.trim();
                                    if !trimmed.is_empty() {
                                        // Copy to clipboard silently (sans déclencher l'OSD clipboard)
                                        if ClipboardService::set_clipboard_content_silent(trimmed) {
                                            println!(
                                                "Transcription copied to clipboard: {trimmed}"
                                            );
                                            OsdEventService::send_event(OsdEvent::SttComplete(
                                                "Speech Copied".to_string(),
                                            ));
                                        } else {
                                            eprintln!("Failed to copy to clipboard");
                                            OsdEventService::send_event(OsdEvent::SttError(
                                                "Copy failed".to_string(),
                                            ));
                                        }
                                    } else {
                                        OsdEventService::send_event(OsdEvent::SttError(
                                            "No speech detected".to_string(),
                                        ));
                                    }
                                    *state.lock().unwrap() = SttState::Idle;
                                }
                                Err(e) => {
                                    let error_msg = format!("Transcription error: {e}");
                                    OsdEventService::send_event(OsdEvent::SttError(
                                        error_msg.clone(),
                                    ));
                                    *state.lock().unwrap() = SttState::Error(error_msg);
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn stop_recording(&self) -> Result<()> {
        let recorder_guard = self.recorder.lock().unwrap();
        if let Some(recorder) = recorder_guard.as_ref() {
            // Ne pas clear le recorder ici, le thread audio le fera après avoir envoyé les samples
            recorder.stop()
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn get_state(&self) -> SttState {
        self.state.lock().unwrap().clone()
    }
}
