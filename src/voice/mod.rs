use crate::{config::Config, utils::get_path};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedStreamConfig};
use std::error::Error;
use std::{
    ops::Neg,
    sync::mpsc::{channel, Receiver},
    time::Instant,
};
use cpal::{FromSample, Sample};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use rustpotter::{Rustpotter, RustpotterConfig, ScoreMode};

pub fn activated_record(config: Config) -> (Result<bool, Box<dyn Error>>, String){
    let path = get_path(config.file_paths.recording_file);
    let settings = config.recording_settings.main_settings;
    let max = get_max(settings.max_secs as i32);

    return (record(max, settings.silent_secs as f32, &path, settings.silence_start as i32, settings.audio_channels as u16, settings.rate as u32, false), path);
}

fn get_max(num : i32) -> Option<i32>{
    return if num > 0{
        Some(num)
    }else{
        None
    }; 
}

pub fn wake_record(config: Config) -> (Result<bool, Box<dyn Error>>, String){
    let path = get_path(config.file_paths.wake_file);
    let settings = config.recording_settings.wake_settings;
    let max = get_max(settings.max_secs as i32);

    return (record(max, settings.silent_secs as f32, &path, settings.silence_start as i32, settings.audio_channels as u16, settings.rate as u32, true), path);
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

pub fn get_rustpotter_config(sample_rate : u32, channels : u16) -> RustpotterConfig{
    let mut rustpotter_config = RustpotterConfig::default();
    rustpotter_config.detector.threshold = 0.45;
    rustpotter_config.detector.avg_threshold = 0.0;
    rustpotter_config.detector.min_scores = 10;
    rustpotter_config.detector.eager = true;
    rustpotter_config.detector.score_ref = 0.22;
    rustpotter_config.fmt.sample_rate = sample_rate as usize;
    rustpotter_config.fmt.sample_format = rustpotter::SampleFormat::F32;
    rustpotter_config.fmt.channels = channels.clone();
    rustpotter_config.detector.score_mode = ScoreMode::Max;
    rustpotter_config.filters.band_pass.enabled = false;
    rustpotter_config.filters.gain_normalizer.enabled = false;
    
    return rustpotter_config;
}

pub fn record(max_seconds : Option<i32>, timeout : f32, path : &str, silence : i32, channels : u16, sample_rate : u32, wake_word : bool) -> Result<bool, Box<dyn Error>> {
    let silence = silence / 2;
    let host = cpal::default_host();

    let device = match host.default_input_device(){
        Some(x) => x,
        None => return Err("Default device not found.".into()),
    };

    let rustpotter_config = get_rustpotter_config(sample_rate, channels);    
    let mut rustpotter = Rustpotter::new(&rustpotter_config)?;
    rustpotter.add_wakeword_from_file("hey", &get_path("voice/sade.rpw".to_string()))?;

    let (sound_sender, sound_receiver) = channel();
    let mut stream_config = device.default_input_config()?;

    stream_config = SupportedStreamConfig::new(
        channels,
        SampleRate(sample_rate),
        *stream_config.clone().buffer_size(),
        stream_config.clone().sample_format(),
    );
    

    let spec = wav_spec_from_config(&stream_config);
    let writer = hound::WavWriter::create(path, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let writer_2 = Arc::clone(&writer);
    println!("Started Recording");

    let stream = device.build_input_stream(
        &stream_config.into(),
        move |data: &[f32], _: &_| {
            sound_sender.send(data.to_owned());
            write_input_data::<f32, f32>(data, &writer_2)
        },
        move |_| {},
        None,
    )?;

    stream.play();
    
    let found = start(
        &sound_receiver,
        silence,
        timeout,
        max_seconds,
        &mut rustpotter,
        wake_word
    )?;

    let _= writer.lock().unwrap().take().unwrap().finalize();

    println!("finished");
    return Ok(found);
}

fn start(
    sound_receiver: &Receiver<Vec<f32>>,
    silence_level: i32,
    pause_length: f32,
    max_seconds : Option<i32>,
    spotter : &mut Rustpotter,
    is_wake : bool
) -> Result<bool, Box<dyn Error>> {
    let mut silence_start = None;
    let mut sound_from_start_till_pause : Vec<f32> = Vec::new();
    let now = Instant::now();
    loop {
        if let Some(max) = max_seconds{
            if now.elapsed().as_secs() >= max as u64 && !is_wake{
                return Ok(false);
            }
        }

        let small_sound_chunk = sound_receiver.recv().unwrap();
        sound_from_start_till_pause.extend(&small_sound_chunk);

        if is_wake && sound_from_start_till_pause.len() >= spotter.get_samples_per_frame(){
            let detection = spotter.process_samples(sound_from_start_till_pause.drain(0..spotter.get_samples_per_frame()).as_slice().into());
            if let Some(detection) = detection {
                println!("{:?}", detection);
                return Ok(true);
            }
        }

        if is_wake{
            continue;
        }

        let sound_as_ints = small_sound_chunk.iter().map(|f| (*f * 1000.0) as i32);
        let max_amplitude = sound_as_ints.clone().max().unwrap_or(0);
        let min_amplitude = sound_as_ints.clone().min().unwrap_or(0);
        let silence_detected = max_amplitude < silence_level && min_amplitude > silence_level.neg();
        
        silence_start = match silence_detected {
            true => match silence_start {
                None => Some(Instant::now()),
                Some(s) => {
                    if s.elapsed().as_secs_f32() > pause_length && !is_wake {
                        return Ok(false);
                    }
                    Some(s)
                }
            }
            false => None
        }
    }
}
