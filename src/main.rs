use voice_assistant_rs::speech_to_text::wake::{WakeWords};
use voice_assistant_rs::speech_to_text::{create_model, run_whisper};
use faster_whisper_rs::WhisperModel as FWhisperModel;
use voice_assistant_rs::llm::{get_model, run_ollama};
use voice_assistant_rs::utils::{get_path, remove_text_in_brackets, write_read_line};
use voice_assistant_rs::voice::{activated_record, wake_record};
use voice_assistant_rs::config::{Config, Root};
use tokio;
use ollama_rs::Ollama;
use std::time::SystemTime;
use voice_assistant_rs::beep::beep;
use simple_transcribe_rs::transcriber::Transcriber;
use natural_tts::*;
use tts::Tts;


async fn set_up_whisper(config : Config) -> (FWhisperModel, Transcriber){
    let whisper = create_model(config.clone()).await;
    let model = config.models.stt_models.main_model.clone();
    let device = config.models.stt_models.device.clone();
    let compute = config.models.stt_models.compute_type.clone();
    let fwhisper = FWhisperModel::new(model, device, compute).unwrap();
    
    return (fwhisper, whisper);
}

fn set_up_tts(config : Config) -> NaturalTts{

    let desc = "A female speaker in fast calming voice in a quiet environment".to_string();
    let model = "parler-tts/parler-tts-mini-expresso".to_string();
    let parler = natural_tts::models::parler::ParlerModel::new(desc, model, false);

    let mut tts_models = natural_tts::NaturalTtsBuilder::default()
        .default_model(natural_tts::Model::Gtts)
        .gtts_model(natural_tts::models::gtts::GttsModel::default())
        .parler_model(parler.unwrap())
        .tts_model(natural_tts::models::tts_rs::TtsModel::default())
        .build().unwrap();

    return tts_models;
}

fn get_config() -> Config{
    return Root::load(get_path("config/config.json".to_string())).unwrap().config;
}

#[tokio::main]
async fn main() {
    let mut ollama = get_model();
    let config = get_config();
    let (fwhisper, whisper) = set_up_whisper(config.clone()).await; 
    let mut tts_models = set_up_tts(config.clone());

    let mut found = false;

    loop{
        if config.clone().general_settings.text_mode{
            let user = write_read_line(">>> ".to_string());
            let now = SystemTime::now(); 
            let ai = run_ollama(user, &mut ollama, &config.clone().models.llm_model.model).await;
            if let Ok(elapsed) = now.elapsed(){
                println!("\nChatbot time: {:?}", elapsed.as_secs());
            }

            if let Ok(ai) = ai{
                println!("\n>>> {}", ai);
                tts_models.say_auto(ai.clone());
                //say(&mut tts_models,ai.clone(), &tts_model);
            }else{
                println!("{}", ai.unwrap_err());
            }
        }else{
            found = match found{
                true => word_found(&whisper, &fwhisper, &mut ollama, &mut tts_models, config.clone()).await,
                false => word_not_found(config.clone()),
            };
        }
    }
}

pub async fn word_found(trans_main : &Transcriber, fwhisper : &FWhisperModel, ollama : &mut Ollama, tts_models : &mut NaturalTts,config : Config) -> bool{
    beep();

    let recording = activated_record(config.clone());
    let path = match recording.0{
        Ok(..) => recording.1,
        Err(..) => return true,
    };
    
    beep();

    let transcript = match run_whisper(trans_main, fwhisper, path.to_string(), true, true){
        Ok(x) => x,
        Err(..) => return true,
    };
    
    if remove_text_in_brackets(transcript.trim()) != ""{
        let ai = run_ollama(remove_text_in_brackets(transcript.trim()).to_string(), ollama, &config.models.llm_model.model).await;

        if let Ok(ai) = ai.clone(){
            println!("\n{}", ai.clone());

            tts_models.say_auto(ai.clone());
        }else{
            println!("\n{}", ai.unwrap_err());
            return false;
        }

        if ai.unwrap().contains_wake_words(vec!["provide".to_string(), "understand".to_string()]){
            return true;
        }
    }   
    return false;
}

pub fn word_not_found(config : Config) -> bool{
    return wake_record(config.clone()).0;
}
