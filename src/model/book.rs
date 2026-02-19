use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSearchResult {
    pub id: Uuid,
    pub reference: String,
    pub title: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub editor: Option<String>,
    pub edition_date: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDetail {
    pub id: Uuid,
    pub reference: String,
    pub title: String,
    pub authors: Vec<String>,
    pub editor: Option<String>,
    pub tags: Vec<String>,
    pub edition_date: Option<String>,
    pub summary: Option<String>,
    pub introduction: Option<String>,
    pub cover_text: Option<String>,
    pub ean: Option<String>,
    pub isbn: Option<String>,
    pub reseller_urls: Vec<ResellerUrl>,
    pub chapter_summaries: Vec<ChapterSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerUrl {
    pub url: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSummary {
    pub chapter_idx: i32,
    pub title: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(default)]
    pub sources: Vec<ChatSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSource {
    pub reference: String,
    pub title: String,
    pub chunk_text: String,
}
