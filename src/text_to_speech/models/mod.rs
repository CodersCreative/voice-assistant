pub mod coqui;
pub mod parler;
pub mod gtts;

pub trait TtsModelTrait {
    fn say(&self, message : String, path : String);
}
