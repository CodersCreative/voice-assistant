use pyo3::{prelude::*, types::PyModule};
use super::TtsModelTrait;

pub struct GttsModel {
    module: Py<PyModule>,
}

impl GttsModel{
    pub fn new() -> Result<Self, String>{
        let m = Python::with_gil(|py|{
            let activators = PyModule::from_code_bound(py, r#"
from gtts import gTTS
def say(message, path):
    tts = gTTS(message)
    tts.save(path)
            "#, "parler.py", "Parler"
            ).unwrap();

            return Self{
                module: activators.unbind(),
            };
        });

        return Ok(m);
    }
}

impl TtsModelTrait for GttsModel{
    fn say(&self, message: String, path : String){
        Python::with_gil(|py|{
            let _ =self.module.getattr(py, "say").unwrap().call1(py, (message, path, ));
        });
    }
}
