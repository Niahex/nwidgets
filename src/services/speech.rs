use anyhow::Result;
use base64::Engine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct SpeechRecognitionService {
    is_recording: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<Vec<i16>>>,
}

impl SpeechRecognitionService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            is_recording: Arc::new(Mutex::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Start recording and transcribing audio
    pub fn start_recording<F>(&self, on_text: F) -> Result<()>
    where
        F: Fn(String) + Send + 'static,
    {
        *self.is_recording.lock().unwrap() = true;
        self.audio_buffer.lock().unwrap().clear();

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;

        println!(
            "[SPEECH] Using input device: {}",
            device.name().unwrap_or_default()
        );
        println!("[SPEECH] Sample rate: {}", sample_rate);

        let is_recording = self.is_recording.clone();
        let audio_buffer = self.audio_buffer.clone();

        // Build the input stream
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream::<f32>(
                &device,
                &config.into(),
                is_recording.clone(),
                audio_buffer.clone(),
            )?,
            cpal::SampleFormat::I16 => self.build_stream::<i16>(
                &device,
                &config.into(),
                is_recording.clone(),
                audio_buffer.clone(),
            )?,
            cpal::SampleFormat::U16 => self.build_stream::<u16>(
                &device,
                &config.into(),
                is_recording.clone(),
                audio_buffer.clone(),
            )?,
            sample_format => {
                anyhow::bail!("Unsupported sample format: {}", sample_format)
            }
        };

        stream.play()?;

        // Spawn a task to periodically send audio to Google for transcription
        let is_recording_clone = is_recording.clone();
        let audio_buffer_clone = audio_buffer.clone();
        std::thread::spawn(move || {
            // Use the shared tokio runtime
            crate::utils::runtime::block_on(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                if !*is_recording_clone.lock().unwrap() {
                    break;
                }

                // Get audio buffer
                let samples: Vec<i16> = {
                    let mut buffer = audio_buffer_clone.lock().unwrap();
                    if buffer.is_empty() {
                        continue;
                    }
                    buffer.drain(..).collect()
                };

                println!("[SPEECH] Transcribing {} samples...", samples.len());

                // Send to Google Speech API
                match Self::transcribe_audio(&samples, sample_rate).await {
                    Ok(text) => {
                        if !text.is_empty() {
                            println!("[SPEECH] ✅ Recognized: '{}'", text);
                            on_text(text);
                        } else {
                            println!("[SPEECH] ⚠️  No text recognized (empty response)");
                        }
                    }
                    Err(e) => {
                        eprintln!("[SPEECH] ❌ Transcription error: {}", e);
                    }
                }
            }
            });
        });

        // Keep the stream alive
        std::mem::forget(stream);

        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&self) {
        *self.is_recording.lock().unwrap() = false;
        println!("[SPEECH] Recording stopped");
    }

    fn build_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        is_recording: Arc<Mutex<bool>>,
        audio_buffer: Arc<Mutex<Vec<i16>>>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
    {
        let channels = config.channels as usize;

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if !*is_recording.lock().unwrap() {
                    return;
                }

                // Convert samples to i16
                let samples: Vec<i16> = data
                    .iter()
                    .step_by(channels) // Take only first channel if stereo
                    .map(|sample| {
                        use cpal::Sample;
                        let float_sample = sample.to_float_sample();
                        let sample_f32: f32 = float_sample.to_sample();
                        (sample_f32 * 32767.0) as i16
                    })
                    .collect();

                // Add to buffer
                audio_buffer.lock().unwrap().extend_from_slice(&samples);
            },
            |err| eprintln!("[SPEECH] Stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    /// Transcribe audio using Google Speech API (unofficial)
    async fn transcribe_audio(samples: &[i16], sample_rate: u32) -> Result<String> {
        // Convert i16 samples to raw PCM bytes (little-endian)
        let pcm_data: Vec<u8> = samples
            .iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();

        // Google Speech API endpoint (unofficial)
        let url = format!(
            "https://www.google.com/speech-api/v2/recognize?output=json&lang=fr-FR&key=AIzaSyBOti4mM-6x9WDnZIjIeyEU21OpBXqWBgw&client=chromium"
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", format!("audio/l16; rate={}", sample_rate))
            .body(pcm_data)
            .send()
            .await?;

        let text = response.text().await?;

        println!("[SPEECH] 📡 Raw response from Google: {:?}", text);

        // Parse JSON response (format: {"result":[{"alternative":[{"transcript":"text","confidence":0.9}]}]})
        // The response might have multiple lines, take the last non-empty one
        for line in text.lines().rev() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(result) = json.get("result") {
                    if let Some(alternatives) = result.get(0).and_then(|r| r.get("alternative")) {
                        if let Some(transcript) = alternatives.get(0).and_then(|a| a.get("transcript")) {
                            if let Some(text) = transcript.as_str() {
                                return Ok(text.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(String::new())
    }

    /// Encode audio samples to FLAC format
    fn encode_flac(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>> {
        use std::io::Cursor;

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;

        for &sample in samples {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;

        // For now, return WAV data (Google also accepts WAV)
        // TODO: Convert to FLAC for smaller size
        Ok(cursor.into_inner())
    }
}
