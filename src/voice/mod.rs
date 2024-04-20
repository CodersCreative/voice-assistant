use crate::{config::*, utils::*};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedBufferSize, SupportedStreamConfig, SupportedStreamConfigsError};
use std::{
    ops::Neg,
    sync::mpsc::{channel, Receiver},
    time::Instant,
};
use cpal::{FromSample, Sample};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use rustpotter::{Rustpotter, RustpotterConfig};

pub fn activated_record(config: Config) -> (Result<bool, String>, String){
    let path = get_path(config.file_paths.recording_file);
    let settings = config.recording_settings.main_settings;
    
    let max = if settings.max_secs > 0{
        Some(settings.max_secs as i32)
    }else{
        None
    };

    return (record(max, settings.silent_secs as f32, &path, settings.silence_start as i32, settings.audio_channels as u16, settings.rate as u32, false), path);
}

pub fn wake_record(config: Config) -> (bool, String){
    let path = get_path(config.file_paths.wake_file);
    let settings = config.recording_settings.wake_settings;
    let max = if settings.max_secs > 0{
        Some(settings.max_secs as i32)
    }else{
        None
    };

    return (match record(max, settings.silent_secs as f32, &path, settings.silence_start as i32, settings.audio_channels as u16, settings.rate as u32, true){
        Ok(x) => x,
        Err(..) => false
    }, path);
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

pub fn record(
    max_seconds : Option<i32>, timeout : f32, path : &str, silence : i32, channels : u16, sample_rate : u32, wake_word : bool,
) -> Result<bool, String> {
    let silence = silence / 2;
    let host = cpal::default_host();
    let device = match host.default_input_device(){
        Some(x) => x,
        None => return Err("No devices.".to_string()),
    };

    let rustpotter_config = if wake_word{
        let mut def = RustpotterConfig::default();
        def.fmt.sample_rate = sample_rate as usize;
        def.fmt.sample_format = rustpotter::SampleFormat::F32;
        def.fmt.channels = channels.clone();
        def.detector.threshold = 0.2;
        Some(def)
    }else{
        None
    };

    let mut rustpotter_model = match rustpotter_config{
        Some(x) => match Rustpotter::new(&x){
            Ok(mut y) => {y.add_wakeword_from_file("wakeword_key", &get_path("voice/alexa.rpw".to_string())).unwrap(); Some(y)},
            Err(e) => return Err(e.to_string()),
        },
        None => None,
    };



    
    let (sound_sender, sound_receiver) = channel();
    let mut stream_config = match device.default_input_config(){
        Ok(x) => x,
        Err(e) => return Err(e.to_string()),
    };

    let spec = wav_spec_from_config(&stream_config);
    let writer = match hound::WavWriter::create(path, spec){
        Ok(x) => x,
        Err(e) => return Err(e.to_string()),
    };
    let writer = Arc::new(Mutex::new(Some(writer)));

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();
    println!("Started Recording");

    stream_config = SupportedStreamConfig::new(
        channels,
        SampleRate(sample_rate),
        *stream_config.clone().buffer_size(),
        stream_config.clone().sample_format(),
    );

    let stream = device.build_input_stream(
        &stream_config.into(),
        move |data: &[f32], _: &_| {
            sound_sender.send(data.to_owned());
            write_input_data::<f32, f32>(data, &writer_2)
        },
        move |err| {},
        None,
    );

    let stream = match  stream {
        Ok(x) => x,
        Err(e) => return Err(e.to_string()),
    };

    return match stream.play() {
        Ok(()) => {
            let found = match start(
                &sound_receiver,
                silence,
                timeout,
                max_seconds,
                &mut rustpotter_model,
            ){
                Ok(x) => x,
                Err(..) => false, 
            };

            let _= writer.lock().unwrap().take().unwrap().finalize();
            return Ok(found);
        }
        Err(err) => {
            Err(err.to_string())
        }
    };
}

fn start(
    sound_receiver: &Receiver<Vec<f32>>,
    silence_level: i32,
    pause_length: f32,
    max_seconds : Option<i32>,
    spotter : &mut Option<Rustpotter>
) -> Result<bool, String> {
    let mut silence_start = None;
    let mut sound_from_start_till_pause : Vec<f32> = Vec::new();
    let now = Instant::now();
    loop {
        if let Some(max) = max_seconds{
            if now.elapsed().as_secs() >= max as u64{
                return Ok(false);
            }
        }
        let small_sound_chunk = sound_receiver.recv().unwrap();
        sound_from_start_till_pause.extend(&small_sound_chunk);
        if let Some(s) = spotter{

            let detection = s.process_samples(sound_from_start_till_pause.clone());
            if let Some(detection) = detection {
                println!("{:?}", detection);
                return Ok(true);
            }
        }
        let sound_as_ints = small_sound_chunk.iter().map(|f| (*f * 1000.0) as i32);
        let max_amplitude = sound_as_ints.clone().max().unwrap_or(0);
        let min_amplitude = sound_as_ints.clone().min().unwrap_or(0);
        let silence_detected = max_amplitude < silence_level && min_amplitude > silence_level.neg();
        if silence_detected {
            match silence_start {
                None => silence_start = Some(Instant::now()),
                Some(s) => {
                    if s.elapsed().as_secs_f32() > pause_length {
                        return Ok(false);
                    }
                }
            }
        } else {
            silence_start = None;
        }
    }
}


