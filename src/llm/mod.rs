use ollama_rs::{
    generation::chat::{
        request::ChatMessageRequest, ChatMessage,
    },
    Ollama,
};
use std::time::Instant;

pub fn get_model() -> Ollama{
    return Ollama::new_default_with_history(50);
}

pub async fn run_ollama(input : String, ollama : &mut Ollama, model : &String) -> Result<String, String>{
    let user_message = ChatMessage::user(input.to_string());
    let now = Instant::now();
    let result = ollama.send_chat_messages_with_history(ChatMessageRequest::new(model.clone(), vec![user_message]), "default".to_string()).await;
    println!("LLM Time: {}", now.elapsed().as_secs());

    if let Ok(result) = result{
        if result.message.is_none(){
            return Err("No Result".to_string());
        }
        let response = result.message.unwrap().content;
        return Ok(response.into());
    }
    return Err("Failed to run ollama.".to_string());
}
