use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crossbeam_channel::Sender;

pub fn spawn_system_audio_capture(
    output_path: &PathBuf,
    _device_name: &str, // Not strictly needed for WASAPI loopback, we can just use default output
) -> Result<Sender<()>, String> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or("Failed to get default output device")?;

    let config = device
        .default_output_config()
        .map_err(|e| format!("Failed to get default output config: {e}"))?;

    let sample_format = config.sample_format();
    match sample_format {
        cpal::SampleFormat::F32 | cpal::SampleFormat::I16 | cpal::SampleFormat::U16 => {}
        _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
    }
    let config: cpal::StreamConfig = config.into();

    let spec = WavSpec {
        channels: config.channels as u16,
        sample_rate: config.sample_rate.0,
        bits_per_sample: if sample_format == cpal::SampleFormat::F32 { 32 } else { 16 },
        sample_format: if sample_format == cpal::SampleFormat::F32 { hound::SampleFormat::Float } else { hound::SampleFormat::Int },
    };


    let writer = WavWriter::create(output_path, spec)
        .map_err(|e| format!("Failed to create WavWriter: {e}"))?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let (tx, rx) = crossbeam_channel::bounded::<()>(1);
    
    let writer_clone = writer.clone();
    
    std::thread::spawn(move || {
        let err_fn = move |err| {
            log::error!("an error occurred on stream: {}", err);
        };

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &_| {
                    if let Ok(mut w) = writer_clone.lock() {
                        if let Some(writer) = w.as_mut() {
                            for &sample in data {
                                let _ = writer.write_sample(sample);
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _: &_| {
                    if let Ok(mut w) = writer_clone.lock() {
                        if let Some(writer) = w.as_mut() {
                            for &sample in data {
                                let _ = writer.write_sample(sample);
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _: &_| {
                    if let Ok(mut w) = writer_clone.lock() {
                        if let Some(writer) = w.as_mut() {
                            for &sample in data {
                                let _ = writer.write_sample((sample as i32 - 32768) as i16);
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => return,
        };

        if let Ok(stream) = stream {
            if stream.play().is_ok() {
                let _ = rx.recv(); // Block until stop signal
            }
            drop(stream);
        }

        if let Ok(mut w) = writer.lock() {
            if let Some(writer) = w.take() {
                let _ = writer.finalize();
            }
        }
    });

    Ok(tx)
}