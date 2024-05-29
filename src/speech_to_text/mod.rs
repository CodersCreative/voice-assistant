pub mod wake;
pub mod faster_whisper;

use simple_transcribe_rs::transcriber::Transcriber;
use whisper_rs::{FullParams, SamplingStrategy};
use simple_transcribe_rs::{transcriber, model_handler};
use crate::config::Config;
use std::time::Instant;
use crate::speech_to_text::faster_whisper::FWhisperModel;

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

pub fn run_whisper(trans : &Transcriber, fwhisper : &FWhisperModel, path : String, use_faster : bool, vad : bool) -> Result<String, String>{
    if use_faster{
        let now = Instant::now();
        let transcript = fwhisper.transcribe(vad, path.clone());
        
        if let Some(transcript) = transcript{
            return Ok(transcript);
        }

        println!("STT Time: {}", now.elapsed().as_secs());
    }

    let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: 1.0 });
    set_params(&mut params);
    
    let now = Instant::now();
    let result = trans.transcribe(&path, Some(params));

    println!("STT Time: {}", now.elapsed().as_secs());

    if let Ok(result) = result{
        println!(">>> {}", result.get_text());
        return Ok(result.get_text().to_string());
    }

    return Err(result.unwrap_err().to_string());
}


