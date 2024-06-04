use voice_assistant_rs::speech_to_text::wake::{WakeWords};
use voice_assistant_rs::speech_to_text::{create_model, run_whisper};
use faster_whisper_rs::WhisperModel as FWhisperModel;
use voice_assistant_rs::llm::{get_model, run_ollama};
use voice_assistant_rs::utils::{get_path, remove_text_in_brackets, write_read_line};
use voice_assistant_rs::voice::{activated_record, wake_record};
use voice_assistant_rs::config::{Config, Root};
use tokio;
use ollama_rs::Ollama;
use std::error::Error;
use std::time::SystemTime;
use voice_assistant_rs::beep::beep;
use simple_transcribe_rs::transcriber::Transcriber;
use natural_tts::*;

async fn set_up_whisper(config : &Config) -> Result<(FWhisperModel, Transcriber), Box<dyn Error>>{
    let whisper = create_model(config.clone()).await;
    let model = config.models.stt_models.main_model.clone();
    let device = config.models.stt_models.device.clone();
    let compute = config.models.stt_models.compute_type.clone();
    let fwhisper = FWhisperModel::new(model, device, compute).unwrap();
    
    return Ok((fwhisper, whisper));
}

fn set_up_tts(config : &Config) -> Result<NaturalTts, Box<dyn Error>>{

    let desc = "A female speaker in fast calming voice in a quiet environment".to_string();
    let model = "parler-tts/parler-tts-mini-expresso".to_string();
    let parler = natural_tts::models::parler::ParlerModel::new(desc, model, false);

    Ok(natural_tts::NaturalTtsBuilder::default()
        .default_model(natural_tts::Model::Gtts)
        .gtts_model(natural_tts::models::gtts::GttsModel::default())
        .parler_model(parler.unwrap())
        .tts_model(natural_tts::models::tts_rs::TtsModel::default())
        .build()?)
}

fn get_config() -> Config{
    return Root::load(get_path("config/config.json".to_string())).unwrap().config;
}

#[tokio::main]
async fn main() -> Result<(()), Box<dyn Error>>{
    let mut ollama = get_model();
    let config = get_config();
    let (fwhisper, whisper) = set_up_whisper(&config).await?; 
    let mut tts_models = set_up_tts(&config)?;

    let mut found = false;

    loop{
        if config.clone().general_settings.text_mode{
            
            let user = match write_read_line(">>> ".to_string()) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Err: {}", e.to_string());
                    continue;
                },
            };

            let now = SystemTime::now(); 
            let ai = match run_ollama(user, &mut ollama, &config.clone().models.llm_model.model).await {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Err: {}", e.to_string());
                    continue;
                },
            };
            if let Ok(elapsed) = now.elapsed(){
                println!("\nChatbot time: {:?}", elapsed.as_secs());
            }
            tts_models.say_auto(ai);
        }else{
            found = match found{
                true => match word_found(&whisper, &fwhisper, &mut ollama, &mut tts_models, config.clone()).await {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("Err: {}", e.to_string());
                        true
                    },
                },
                false => match word_not_found(config.clone()){
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("Err: {}", e.to_string());
                        false
                    },

                },
            };
        }
    }

    Ok(())
}

pub async fn word_found(trans_main : &Transcriber, fwhisper : &FWhisperModel, ollama : &mut Ollama, tts_models : &mut NaturalTts,config : Config) -> Result<bool, Box<dyn Error>>{
    let _ = beep();

    let recording = activated_record(config.clone());
    let _ = recording.0?;
    let path = recording.1.as_str();

    let _ = beep();

    let transcript = run_whisper(trans_main, fwhisper, path, true, true)?;
    let rmb = remove_text_in_brackets(transcript.trim())?;
    
    if !rmb.is_empty(){
        let ai = run_ollama(rmb.to_string(), ollama, &config.models.llm_model.model).await?;
        let _ = tts_models.say_auto(ai.clone());

        if ai.contains_wake_words(vec!["provide".to_string(), "understand".to_string()]){
            return Ok(true);
        }
    }   
    
    Ok(false)
}

pub fn word_not_found(config : Config) -> Result<bool, Box<dyn Error>>{
    wake_record(config.clone()).0
}
