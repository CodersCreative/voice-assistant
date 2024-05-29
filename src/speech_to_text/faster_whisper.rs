use pyo3::{prelude::*, types::PyModule};
use crate::config::Config;

pub struct FWhisperModel {
    module: Py<PyModule>,
    model: Py<pyo3::PyAny>,
}

impl FWhisperModel{
    pub fn new(config : Config) -> Result<Self, String>{
        let m = Python::with_gil(|py|{
            let activators = PyModule::from_code_bound(py, r#"

from faster_whisper import WhisperModel
import os

def new_model(size, device, compute):
    return WhisperModel(size, device=device, compute_type=compute)

def transcribe_audio(model, path, vad=True):
    segments, _ = model.transcribe(audio=path, beam_size=5, vad_filter=vad) #
    segments = list(segments)
    transcript = ""
    
    for segment in segments:
        transcript += segment.text

    print(f"Path: {path}, Transcript: {transcript}")
    os.remove(path)
    return transcript
            "#, "whisper.py", "Whisper"
            ).unwrap();
            let args = (config.models.stt_models.main_model.clone(), config.models.stt_models.device.clone(), config.models.stt_models.compute_type.clone());
            let model = activators.getattr("new_model").unwrap().call1(args).unwrap().unbind();
            return Self{
                module: activators.unbind(),
                model,
            };
        });
        
        return Ok(m);
    }

    pub fn transcribe(&self, vad : bool, path : String) -> Option<String>{
        let transcript = Python::with_gil(|py|{
            let args = (self.model.clone(), path, vad);
            let transcript = self.module.getattr(py, "transcribe_audio").unwrap().call1(py, args);
            
            if let Err(e) = &transcript{
                println!("Just Error");
                println!("{}", e.to_string());
            }
            
            return transcript.unwrap();
        });
        
        return Some(transcript.to_string());
    }
}
