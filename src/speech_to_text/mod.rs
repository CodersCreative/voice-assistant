pub mod wake;

use simple_transcribe_rs::transcriber::Transcriber;
use whisper_rs::{FullParams, SamplingStrategy};
use simple_transcribe_rs::{transcriber, model_handler};
use crate::config::Config;
use std::{error::Error, time::Instant};
use faster_whisper_rs::WhisperModel as FWhisperModel;

pub async fn create_model(config : Config) -> Transcriber{
    let m =  model_handler::ModelHandler::new(&config.models.stt_models.main_model, "models/").await;
    return transcriber::Transcriber::new(m);
}

pub fn set_params(params : &mut FullParams){    
    params.set_n_threads(2);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
}

pub fn run_whisper(trans : &Option<Transcriber>, fwhisper : &Option<FWhisperModel>, path : &str, use_faster : bool, vad : bool) -> Result<String, Box<dyn Error>>{
    if use_faster{
        if let Some(fwhisper) = fwhisper{
            let now = Instant::now();
            let transcript = fwhisper.transcribe(path.to_string());
            
            if let Ok(transcript) = transcript{
                return Ok(transcript.to_string());
            }

            println!("STT Time: {}", now.elapsed().as_secs());
        }
    }

    let trans = match trans{
        Some(x) => x,
        None => return Err("No STT Model".into()),
    };

    let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: 1.0 });
    set_params(&mut params);
    
    let now = Instant::now();
    let result = trans.transcribe(path, Some(params))?;

    println!("STT Time: {}", now.elapsed().as_secs());

    println!(">>> {}", result.get_text());
    return Ok(result.get_text().to_string());
}


