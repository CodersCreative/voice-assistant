use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use serde_json;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub config: Config,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub models: Models,
    #[serde(rename = "file_paths")]
    pub file_paths: FilePaths,
    #[serde(rename = "wake_words")]
    pub wake_words: Vec<String>,
    #[serde(rename = "recording_settings")]
    pub recording_settings: RecordingSettings,
    #[serde(rename = "general_settings")]
    pub general_settings: GeneralSettings,
}

impl Root{
    pub fn load(path : String) -> Result<Self, Box<dyn Error>>{
        let mut reader = File::open(path)?;
        let mut data = String::new();
        let _ = reader.read_to_string(&mut data)?;

        Ok(serde_json::from_str(&data)?)
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Models {
    #[serde(rename = "stt_models")]
    pub stt_models: SttModels,
    #[serde(rename = "tts_model")]
    pub tts_model: TtsModel,
    #[serde(rename = "llm_model")]
    pub llm_model: LlmModel,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SttModels {
    #[serde(rename = "wake_model")]
    pub wake_model: String,
    #[serde(rename = "main_model")]
    pub main_model: String,
    pub device: String,
    #[serde(rename = "compute_type")]
    pub compute_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsModel {
    pub model: String,
    #[serde(rename = "progress_bar")]
    pub progress_bar: bool,
    pub gpu: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmModel {
    pub model: String,
    pub template: String,
    pub verbose: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePaths {
    #[serde(rename = "wake_file")]
    pub wake_file: String,
    #[serde(rename = "recording_file")]
    pub recording_file: String,
    #[serde(rename = "output_file")]
    pub output_file: String,
    #[serde(rename = "beep_file")]
    pub beep_file: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSettings {
    pub general: General,
    #[serde(rename = "wake_settings")]
    pub wake_settings: WakeSettings,
    #[serde(rename = "main_settings")]
    pub main_settings: MainSettings,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct General {
    #[serde(rename = "chunk_size")]
    pub chunk_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeSettings {
    pub rate: i64,
    #[serde(rename = "silent_secs")]
    pub silent_secs: i64,
    #[serde(rename = "max_secs")]
    pub max_secs: i64,
    #[serde(rename = "audio_channels")]
    pub audio_channels: i64,
    #[serde(rename = "vad_filter")]
    pub vad_filter: bool,
    #[serde(rename = "silence_start")]
    pub silence_start: i64,
    #[serde(rename = "chunk_size")]
    pub chunk_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainSettings {
    pub rate: i64,
    #[serde(rename = "silent_secs")]
    pub silent_secs: i64,
    #[serde(rename = "max_secs")]
    pub max_secs: i64,
    #[serde(rename = "audio_channels")]
    pub audio_channels: i64,
    #[serde(rename = "vad_filter")]
    pub vad_filter: bool,
    #[serde(rename = "silence_start")]
    pub silence_start: i64,
    #[serde(rename = "chunk_size")]
    pub chunk_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    #[serde(rename = "text_mode")]
    pub text_mode: bool,
    #[serde(rename = "play_beep")]
    pub play_beep: bool,
}
