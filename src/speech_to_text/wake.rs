use crate::{config::*, utils::read_wav_file, speech_to_text::run_whisper};
use regex::Regex;
use simple_transcribe_rs::transcriber::Transcriber;

pub fn check_word(trans : &Transcriber, path : String, config : Config) -> bool{    
    let transcript = match run_whisper(trans, path, true){
        Ok(x) => x,
        Err(..) => return false,
    };
    
    let wake_words = config.wake_words;  // Assuming get_config() returns a HashMap
    let is_activated = transcript.contains_wake_words(wake_words);
    return is_activated;
}

pub trait WakeWords{
    fn contains_wake_words(&self, words : Vec<String>) -> bool;
}

impl WakeWords for String{
    fn contains_wake_words(&self, words : Vec<String>) -> bool{
        return words.iter().any(|word| self.trim().to_lowercase().contains(&word.to_lowercase()));
    }
}

#[test]
fn test_wake_words(){
    let wake_words = vec!["Sade".to_string(), "Said".to_string()];
    let sentence1 = "How are you doing today".to_string();
    let sentence2 = "Said, how are you doing today".to_string();
    let sentence3 = "sade, how are you doing todat".to_string();

    assert_eq!(sentence1.contains_wake_words(wake_words.clone()), false);
    assert_eq!(sentence2.contains_wake_words(wake_words.clone()), true);
    assert_eq!(sentence3.contains_wake_words(wake_words.clone()), true);
}
