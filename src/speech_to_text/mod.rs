use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use crate::{config::Config, utils::get_path};
use std::time::Instant;
use self::wake::WakeWords;
pub mod wake;

pub fn create_model(config : Config) -> (WhisperContext, WhisperContext){
    let whisper_tiny = get_path("speech_to_text/ggml-tiny.en-q8_0.bin".to_string());
    let mut whisper_base = get_path("speech_to_text/ggml-base.en-q5_1.bin".to_string());

    //let whisper_tiny = get_path("speech_to_text/ggml-base.en-q5_1.bin".to_string());
    //let mut whisper_base = get_path("speech_to_text/ggml-base-fp16.bin".to_string());
    
    let have_base = config.models.stt_models.main_model.contains_wake_words(vec!["base".to_string()]);
    
    let gpu = match config.models.stt_models.compute_type.to_lowercase().as_str(){
        "cpu" => false,
        _ => true,
    }; 
    // load a context and model
    return (
        WhisperContext::new_with_params(
            &whisper_tiny,
            WhisperContextParameters{use_gpu:gpu}
        ).expect("failed to load model"),
        match have_base{
            true => WhisperContext::new_with_params(
                &whisper_base,
                WhisperContextParameters{use_gpu:gpu}
            ).expect("failed to load model"),
            false => WhisperContext::new_with_params(
                &whisper_tiny,
                WhisperContextParameters{use_gpu:gpu}
            ).expect("failed to load model"),
        },
    );
}

pub fn set_params(params : &mut FullParams){    
    
    params.set_n_threads(2);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
}

pub fn run_whisper(context : &mut WhisperContext, audio_data : &Vec<f32>, is_beam : bool) -> Result<String, String>{
    let state = context.create_state();
    
    let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: 1.0 });

    //if !is_beam{
        //params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    //}
    set_params(&mut params);
    
    if let Ok(mut state) = state{
        let now = Instant::now(); 
        let r = state.full(params, audio_data);
         if r.is_err(){
             return Err("Failed to run.".to_string());
         }
         println!("STT time : {}", now.elapsed().as_secs());
         
         let num_segments = match state.full_n_segments(){
             Ok(x) => x,
             Err(e) => return Err("Failed to get segments".to_string()),
         };

         let mut text = String::new();

         for i in 0..num_segments{
             let segment = state.full_get_segment_text(i);
             if let Ok(segment) = segment{
                 text.push_str(&segment);
             }
         }

         println!(">>> {}", text);

         return Ok(text);
    }
    
    return Err("Failed to run.".to_string());
}


