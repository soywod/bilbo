use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct BookFrontmatter {
    pub reference: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    pub editor: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub edition_date: Option<String>,
    pub summary: Option<String>,
    pub introduction: Option<String>,
    pub cover_text: Option<String>,
    pub ean: Option<String>,
    pub isbn: Option<String>,
    #[serde(default)]
    pub reseller_paper_urls: Vec<String>,
    #[serde(default)]
    pub reseller_digital_urls: Vec<String>,
}

pub struct ParsedBook {
    pub frontmatter: BookFrontmatter,
    pub content: String,
    pub hash: String,
}

/// Parse a markdown file: extract YAML frontmatter and content, compute hash.
pub fn parse_markdown(raw: &str) -> Result<ParsedBook, String> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(raw);

    let frontmatter: BookFrontmatter = result
        .data
        .ok_or("missing YAML frontmatter")?
        .deserialize()
        .map_err(|e| format!("invalid frontmatter: {e}"))?;

    let content = result.content.trim().to_string();

    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let hash = hex::encode(hasher.finalize());

    Ok(ParsedBook {
        frontmatter,
        content,
        hash,
    })
}

pub struct Chapter {
    pub title: Option<String>,
    pub text: String,
}

/// Split markdown content into chapters based on heading detection.
pub fn extract_chapters(content: &str) -> Vec<Chapter> {
    let parser = Parser::new(content);
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_text = String::new();
    let mut in_heading = false;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) if level == HeadingLevel::H1 || level == HeadingLevel::H2 => {
                // Save previous chapter if it has content
                if !current_text.trim().is_empty() || current_title.is_some() {
                    chapters.push(Chapter {
                        title: current_title.take(),
                        text: current_text.trim().to_string(),
                    });
                    current_text = String::new();
                }
                in_heading = true;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) if in_heading => {
                in_heading = false;
                current_title = Some(heading_text.trim().to_string());
            }
            Event::Text(text) if in_heading => {
                heading_text.push_str(&text);
            }
            Event::Text(text) => {
                current_text.push_str(&text);
                current_text.push(' ');
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    // Push remaining content
    if !current_text.trim().is_empty() || current_title.is_some() {
        chapters.push(Chapter {
            title: current_title,
            text: current_text.trim().to_string(),
        });
    }

    // If no chapters were found, treat entire content as one chapter
    if chapters.is_empty() {
        chapters.push(Chapter {
            title: None,
            text: content.to_string(),
        });
    }

    chapters
}

pub struct Chunk {
    pub chapter_idx: usize,
    pub chapter_title: Option<String>,
    pub chunk_index: usize,
    pub text: String,
}

/// Chunk text into ~2000 character segments with 400 character overlap.
pub fn chunk_text(chapters: &[Chapter]) -> Vec<Chunk> {
    let chunk_size = 2000;
    let overlap = 400;
    let mut chunks = Vec::new();

    for (chapter_idx, chapter) in chapters.iter().enumerate() {
        let text = &chapter.text;
        if text.is_empty() {
            continue;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk_text: String = chars[start..end].iter().collect();

            chunks.push(Chunk {
                chapter_idx,
                chapter_title: chapter.title.clone(),
                chunk_index,
                text: chunk_text,
            });

            chunk_index += 1;
            if end >= chars.len() {
                break;
            }
            start += chunk_size - overlap;
        }
    }

    chunks
}

/// Convert a markdown string to HTML.
pub fn markdown_to_html(md: &str) -> String {
    let parser = Parser::new_ext(md, Options::empty());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
