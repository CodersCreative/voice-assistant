use voice_assistant_rs::speech_to_text::wake::{WakeWords};
use voice_assistant_rs::speech_to_text::{create_model, run_whisper, faster_whisper};
use voice_assistant_rs::llm::{get_model, run_ollama};
use voice_assistant_rs::text_to_speech::{say, Model, TtsModels};
use voice_assistant_rs::utils::{get_path, remove_text_in_brackets, write_read_line};
use voice_assistant_rs::voice::{activated_record, wake_record};
use voice_assistant_rs::config::{Config, Root};
use tokio;
use ollama_rs::Ollama;
use std::time::SystemTime;
use voice_assistant_rs::beep::beep;
use simple_transcribe_rs::transcriber::Transcriber;


async fn set_up_whisper(config : Config) -> (faster_whisper::FWhisperModel, Transcriber){
    let whisper = create_model(config.clone()).await;
    let fwhisper = faster_whisper::FWhisperModel::new(config.clone()).unwrap();
    
    return (fwhisper, whisper);
}

fn set_up_tts(config : Config) -> (TtsModels ,Model){
    let tts_model = Model::Gtts;
    let mut tts_models = TtsModels::new(config.clone());

    return (tts_models, tts_model);
}

fn get_config() -> Config{
    return Root::load(get_path("config/config.json".to_string())).unwrap().config;
}

#[tokio::main]
async fn main() {
    let mut ollama = get_model();
    let config = get_config();
    let (fwhisper, whisper) = set_up_whisper(config.clone()).await; 
    let (mut tts_models, tts_model) = set_up_tts(config.clone());

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
                say(&mut tts_models,ai.clone(), &tts_model);
            }else{
                println!("{}", ai.unwrap_err());
            }
        }else{
            found = match found{
                true => word_found(&whisper, &fwhisper, &mut ollama, &mut tts_models, config.clone(), &tts_model).await,
                false => word_not_found(config.clone()),
            };
        }
    }
}

pub async fn word_found(trans_main : &Transcriber, fwhisper : &faster_whisper::FWhisperModel, ollama : &mut Ollama, tts_models : &mut TtsModels,config : Config, tts_model : &Model) -> bool{
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
            say(tts_models, ai, tts_model);
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
