pub mod parser;
pub mod request;
pub mod response;
pub mod session;
use std::str::FromStr;

use parser::MarkdownToParts;
use regex::Regex;
use request::Part;
use session::Session;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct SessionManager {
    session: Session,
}

#[wasm_bindgen]
impl SessionManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            session: Session::new(10),
        }
    }
    #[wasm_bindgen]
    pub async fn to_parts(prompt: &str) -> String {
        let regex = Regex::from_str(r"(?s)!\[.*?].?\((.*?)\)|https?://([^\s]+\.pdf)\b").unwrap();
        let parser = MarkdownToParts::from_regex(prompt, regex, |url| {
            let extention = url.split(".").last().unwrap();
            if extention.to_lowercase() == "pdf" {
                "application/pdf".into()
            } else {
                format!("image/{extention}")
            }
        })
        .await;
        serde_json::to_string(&parser.process()).unwrap()
    }
    #[wasm_bindgen]
    pub fn ask(&mut self, parts: &str) {
        let parts: Vec<Part> = serde_json::from_str(parts).unwrap();
        self.session.ask(parts);
    }
    #[wasm_bindgen]
    pub fn add_reply(&mut self, parts: &str) {
        let parts: Vec<Part> = serde_json::from_str(parts).unwrap();
        self.session.reply(parts);
    }
    #[wasm_bindgen]
    pub fn get_session(&self) -> String {
        serde_json::to_string(&self.session).unwrap()
    }
}
