use voice_assistant_rs::config;
use voice_assistant_rs::speech_to_text::wake::check_word;
use voice_assistant_rs::speech_to_text::wake::WakeWords;
use voice_assistant_rs::speech_to_text::*;
use voice_assistant_rs::llm::*;
use voice_assistant_rs::text_to_speech::*;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};
use voice_assistant_rs::utils::*;
use voice_assistant_rs::voice::*;
use voice_assistant_rs::config::*;
use tokio;
use ollama_rs::Ollama;
use std::time::{Duration, SystemTime};
use tts::Tts;
use voice_assistant_rs::beep::beep;
use simple_transcribe_rs::transcriber::Transcriber;
#[tokio::main]
async fn main() {
    let mut ollama = get_model();
    let mut tts_model = get_tts();

    let mut config_path = get_path("config/config.json".to_string());
    // The WAV file we're recording to.
    //

    let config = Root::load(config_path).unwrap().config;
    
    let mut whisper = create_model(config.clone()).await;

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
                say(&mut tts_model, ai.clone());
            }else{
                println!("{}", ai.unwrap_err());
            }
        }else{
            if found{
                found = word_found(&whisper, &mut ollama, &mut tts_model, config.clone()).await;
            }else{
                found = word_not_found(config.clone());
            }
        }
    }
}

pub async fn word_found(trans_main : &Transcriber, ollama : &mut Ollama, model : &mut Tts, config : Config) -> bool{
    beep();

    let recording = activated_record(config.clone());
    let path = match recording.0{
        Ok(x) => recording.1,
        Err(..) => return true,
    };
    
    beep();

    let transcript = match run_whisper(trans_main, path.to_string(), false){
        Ok(x) => x,
        Err(..) => return true,
    };
    
    if remove_text_in_brackets(transcript.trim()) != ""{
        let ai = run_ollama(remove_text_in_brackets(transcript.trim()).to_string(), ollama, &config.models.llm_model.model).await;

        if let Ok(ai) = ai.clone(){
            println!("\n{}", ai.clone());
            say(model, ai.clone());
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
