use std::io::{self, Write};
use hound::WavReader;
use std::error::Error;
use regex::Regex;

pub fn remove_text_in_brackets(text: &str) -> Result<String, Box<dyn Error>> {
    let re = Regex::new(r"[\[\]\(\)]+")?;
    Ok(re.replace_all(text, "").to_string())
}

pub fn read_input() -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
}

pub fn write_read(message: String) -> Result<String, Box<dyn Error>> {
    println!("{}", message);
    read_input()
}

pub fn write_read_line(message: String) -> Result<String, Box<dyn Error>>{
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

pub fn read_wav_file(path: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let mut reader = WavReader::open(path)?;
    let mut f32_samples: Vec<f32> = Vec::new();

    reader.samples::<f32>().for_each(|s| {
        let sample = s.unwrap() as f32;
        f32_samples.push(sample);
    });

    Ok(f32_samples)
}

pub fn get_path(path : String) -> String{
    let mut new_path = env!("CARGO_MANIFEST_DIR").to_string();
    new_path.push_str(&format!("/src/{}", path));
    return new_path;
}
