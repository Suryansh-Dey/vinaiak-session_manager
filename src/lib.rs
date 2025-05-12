pub mod request;
pub mod response;
pub mod session;

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
    #[wasm_bindgen]
    pub fn get_last_reply(&self) -> String {
        self.session.get_last_message_text("").unwrap()
    }
}
