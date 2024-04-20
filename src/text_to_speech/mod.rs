use tts::*;
use text_splitter::{Characters, TextSplitter};
use tts_rust::{ tts::GTTSClient, languages::Languages };
use online::check;
use crate::beep::beep;

pub fn get_tts() -> Tts{
    let model = Tts::default();
    return model.unwrap()
}

pub fn say(model : &mut Tts, message : String){
    let is_online = check(Some(1)).is_ok();
    if is_online{
        say_gtts(message.clone());
    }else{
        say_tts(model, message.clone())
    }
}

pub fn say_tts(model : &mut Tts, message: String){
    let is_speaking = model.is_speaking();
    if let Ok(speaking) = is_speaking{
        if speaking{
            return;
        }
    }

    let _ = model.speak(message, false);

}

fn say_gtts(message : String) {
    let mut narrator: GTTSClient = GTTSClient {
        volume: 1.0, 
        language: Languages::English, // use the Languages enum
        tld: "com",
    };
    
    let messages = cut_to_size_gtts(&message);
    
    for message in messages{
        let g = narrator.speak(message);
        if g.is_err(){
            println!("{}", g.unwrap_err());
        }
    }
}

fn cut_to_size_gtts(message: &String) -> Vec<&str>{
    let max_characters = 100;
    
    let splitter = TextSplitter::default()
        .with_trim_chunks(true);

    let chunks = splitter.chunks(&message, max_characters).collect::<Vec<&str>>();
    
    return chunks;
}
