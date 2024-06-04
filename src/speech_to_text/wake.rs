pub trait WakeWords{
    fn contains_wake_words(&self, words : Vec<String>) -> bool;
}

impl WakeWords for String{
    fn contains_wake_words(&self, words : Vec<String>) -> bool{
        return words.iter().any(|word| self.trim().to_lowercase().contains(&word.to_lowercase()));
    }
}

impl WakeWords for str{
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
