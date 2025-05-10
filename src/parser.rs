use base64::{Engine, engine::general_purpose::STANDARD};
use futures::future::join_all;
use gloo_net::http::Request;
use regex::Regex;

pub struct MatchedFiles {
    pub index: usize,
    pub length: usize,
    pub mime_type: Option<String>,
    pub base64: Option<String>,
}
/// # Panics
/// `regex` must have a Regex with atleast 1 capture group with file URL as first capture group, else it PANICS
/// # Arguments
/// `guess_mime_type` is used to detect mimi_type of URL pointing to file system or web resource
/// with no "Content-Type" header.
pub async fn get_file_base64s(
    markdown: impl AsRef<str>,
    regex: Regex,
    guess_mime_type: fn(url: &str) -> String,
) -> Vec<MatchedFiles> {
    let mut tasks: Vec<_> = Vec::new();

    for file in regex.captures_iter(markdown.as_ref()) {
        let capture = file.get(0).unwrap();
        let url = file[1].to_string();
        tasks.push((async |capture: regex::Match<'_>, url: String| {
            let (mime_type, base64) = if url.starts_with("https://") || url.starts_with("http://") {
                println!("trying to make request to {url}");
                let response = Request::get(&url).send().await;
                println!("Done!");
                match response {
                    Ok(response) => {
                        let mime_type = response
                            .headers()
                            .get("Content-Type")
                            .map(|mime| mime.to_string());

                        let base64 = response
                            .binary()
                            .await
                            .ok()
                            .map(|bytes| STANDARD.encode(bytes));
                        let mime_type = match base64 {
                            Some(_) => mime_type.or_else(|| Some(guess_mime_type(&url))),
                            None => None,
                        };
                        (mime_type, base64)
                    }
                    Err(_) => (None, None),
                }
            } else {
                println!("Ignored file path url in WASM: {url}");
                (None, None)
            };
            MatchedFiles {
                index: capture.start(),
                length: capture.len(),
                mime_type,
                base64,
            }
        })(capture, url));
    }
    join_all(tasks).await
}
//parser
use super::request::*;

pub struct MarkdownToParts<'a> {
    base64s: Vec<MatchedFiles>,
    markdown: &'a str,
}
impl<'a> MarkdownToParts<'a> {
    ///# Panics
    /// `regex` must have a Regex with atleast 1 capture group with file URL as first capture group, else it PANICS.
    /// # Arguments
    /// `guess_mime_type` is used to detect mimi_type of URL pointing to file system or web resource
    /// with no "Content-Type" header.
    /// # Example
    /// ```ignore
    /// from_regex("Your markdown string...", Regex::new(r"(?s)!\[.*?].?\((.*?)\)").unwrap(), |_| "image/png".to_string())
    /// ```
    pub async fn from_regex(
        markdown: &'a str,
        regex: Regex,
        guess_mime_type: fn(url: &str) -> String,
    ) -> Self {
        println!("Dikh rha hai?");
        Self {
            base64s: get_file_base64s(markdown, regex, guess_mime_type).await,
            markdown,
        }
    }
    ///Converts markdown to parts considering `![image](link)` means Gemini will be see the images too. `link` can be URL or file path.  
    /// `guess_mime_type` is used to detect mimi_type of URL pointing to file system or web resource
    /// with no "Content-Type" header.
    /// # Example
    /// ```ignore
    /// new("Your markdown string...", |_| "image/png".to_string())
    /// ```
    pub async fn new(markdown: &'a str, guess_mime_type: fn(url: &str) -> String) -> Self {
        let image_regex = Regex::new(r"(?s)!\[.*?].?\((.*?)\)").unwrap();
        Self {
            base64s: get_file_base64s(markdown, image_regex, guess_mime_type).await,
            markdown,
        }
    }
    pub fn process(mut self) -> Vec<Part> {
        let mut parts: Vec<Part> = Vec::new();
        let mut removed_length = 0;
        for file in self.base64s {
            if let MatchedFiles {
                index,
                length,
                mime_type: Some(mime_type),
                base64: Some(base64),
            } = file
            {
                let end = index + length - removed_length;
                let text = &self.markdown[..end];
                parts.push(Part::text(text.to_string()));
                parts.push(Part::inline_data(InlineData::new(mime_type, base64)));

                self.markdown = &self.markdown[end..];
                removed_length += end;
            }
        }
        if self.markdown.len() != 0 {
            parts.push(Part::text(self.markdown.to_string()));
        }
        parts
    }
}
