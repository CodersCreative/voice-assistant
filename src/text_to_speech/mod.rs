pub mod models;

use rodio;
use rodio::buffer::SamplesBuffer;
use tts::Tts;
use text_splitter::TextSplitter;
use online::check;
use crate::text_to_speech::models::TtsModelTrait;
use crate::text_to_speech::models::{coqui, parler, gtts};
use rodio::{Decoder, Sink, OutputStream};
use crate::{config::Config, utils::get_path};

pub fn get_tts() -> Tts{
    let model = Tts::default();
    return model.unwrap()
}

pub struct TtsModels{
    tts_model : Tts,
    parler_model : parler::ParlerModel,
    coqui_model : coqui::CoquiModel,
    gtts_model : gtts::GttsModel,
}

impl TtsModels{
    pub fn new(config : Config) -> Self{
        let ttsm = get_tts();
        let parlerm = parler::ParlerModel::new().unwrap();
        let coquim = coqui::CoquiModel::new(config.models.tts_model.model.clone()).unwrap();
        let gttsm = gtts::GttsModel::new().unwrap();
        
        return Self{
            tts_model : ttsm,
            parler_model : parlerm,
            coqui_model : coquim,
            gtts_model : gttsm,
        }
    }
}

pub enum Model {
    Coqui,
    Parler,
    TTS,
    Gtts
}

pub fn play_model<T : TtsModelTrait>(model : &T, message : String){
        let path = "text_to_speech/output.wav";
        let actual = get_path(path.to_string());
        std::fs::remove_file(actual.clone());
        model.say(message.clone(), actual.clone());
        play_wav_file(&actual);
        std::fs::remove_file(actual);
}

pub fn say(models : &mut TtsModels, message : String, model : &Model){
    let is_online = check(Some(1)).is_ok();

    let gtts_fn = ||{
        match is_online{
            true => play_model(&models.gtts_model, message.clone()),
            false => play_model(&models.parler_model, message.clone()),
        }
    };

    match model{
        Model::Coqui => play_model(&models.coqui_model, message),
        Model::TTS => say_tts(&mut models.tts_model, message),
        Model::Parler => play_model(&models.parler_model, message),
        _ => gtts_fn(), 
    }
}



pub fn say_tts(model : &mut Tts, message: String){
    let is_speaking = model.is_speaking();
    if let Ok(speaking) = is_speaking{
        if speaking{
            println!("Playing");
            return;
        }
    }

    let _ = model.speak(message, false);

}

fn play_audio(data: Vec<f32>, rate : u32){
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let source = SamplesBuffer::new(1, rate, data);
    let sink = rodio::Sink::try_new(&handle).unwrap();

    sink.append(source);

    sink.sleep_until_end();
}


fn play_wav_file(path: &str) {
    let file = std::fs::File::open(path);

    if let Ok(file) = file{
        let decoder = Decoder::new(file).unwrap();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(decoder);
        sink.sleep_until_end();
    }

}

fn cut_to_size(message: &String) -> Vec<&str>{
    let max_characters = 100;
    
    let splitter = TextSplitter::default()
        .with_trim_chunks(true);

    let chunks = splitter.chunks(&message, max_characters).collect::<Vec<&str>>();
    
    return chunks;
}
