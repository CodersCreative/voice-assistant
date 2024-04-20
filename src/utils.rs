
use std::fs::File;
use std::i16;
use std::io::{self, Write};
use hound::{WavReader, WavSamples};
use std::error::Error;
use regex::Regex;

pub fn remove_text_in_brackets(text: &str) -> String {
  let re = Regex::new(r"[\[\]\(\)]+").unwrap();
  re.replace_all(text, "").to_string()
}

pub fn read_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    return input;
}

pub fn write_read(message: String) -> String {
    println!("{}", message);
    return read_input();
}

pub fn write_read_line(message: String) -> String{
    print!("{}", message);
    io::stdout().flush().unwrap();  // Flush to display the prompt
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input;
}

pub fn read_wav_file(path: &str) -> Result<Vec<f32>, Box<dyn Error>> {
  let mut reader = WavReader::open(path).unwrap();

  //let samples = reader.samples();
  let mut f32_samples: Vec<f32> = Vec::new();

  // Convert i16 samples to f32
    reader.samples::<f32>()
                        .for_each(|s| {
        let sample = s.unwrap() as f32;
        f32_samples.push(sample);
    });
  
    Ok(f32_samples)
}

pub fn get_path(path : String) -> String{
    let mut new_path = env!("CARGO_MANIFEST_DIR").to_string();
    // The WAV file we're recording to.
    new_path.push_str(&format!("/src/{}", path));
    return new_path;
}
