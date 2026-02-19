use serde::{Deserialize, Serialize};

const MISTRAL_EMBED_URL: &str = "https://api.mistral.ai/v1/embeddings";
const MISTRAL_CHAT_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const EMBED_MODEL: &str = "mistral-embed";
const CHAT_MODEL: &str = "mistral-small-latest";

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

/// Generate embeddings for a batch of texts (max 16 per call).
pub async fn embed_texts(
    client: &reqwest::Client,
    api_key: &str,
    texts: &[String],
) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(vec![]);
    }

    let mut all_embeddings = Vec::new();

    for batch in texts.chunks(16) {
        let request = EmbedRequest {
            model: EMBED_MODEL.to_string(),
            input: batch.to_vec(),
        };

        let resp = client
            .post(MISTRAL_EMBED_URL)
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("embed request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("embed API error {status}: {body}"));
        }

        let result: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| format!("embed parse error: {e}"))?;

        for data in result.data {
            all_embeddings.push(data.embedding);
        }
    }

    Ok(all_embeddings)
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMsg>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMsg {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMsg,
}

/// Generate a summary for text using Mistral chat.
pub async fn generate_summary(
    client: &reqwest::Client,
    api_key: &str,
    text: &str,
) -> Result<String, String> {
    let system = "Tu es un assistant qui rédige des résumés factuels de livres. \
        Tes résumés doivent être objectifs et concis. \
        Ne commence jamais par des phrases comme « Voici un résumé », « Ce texte parle de », etc. \
        Commence directement par le contenu du résumé. \
        Maximum 5 phrases.";

    let prompt = format!(
        "Résume le texte suivant en français en 5 phrases maximum :\n\n{text}"
    );

    chat_completion(client, api_key, &[("system", system), ("user", &prompt)]).await
}

/// Generate chapter summaries for a list of chapters.
pub async fn generate_chapter_summaries(
    client: &reqwest::Client,
    api_key: &str,
    chapters: &[(Option<String>, String)],
) -> Result<Vec<String>, String> {
    let system = "Tu es un assistant qui rédige des résumés factuels de chapitres de livres. \
        Tes résumés doivent être objectifs et concis. \
        Ne commence jamais par des phrases comme « Voici un résumé », « Ce chapitre parle de », etc. \
        Commence directement par le contenu du résumé. \
        Maximum 3 phrases.";

    let mut summaries = Vec::new();

    for (title, text) in chapters {
        if text.trim().is_empty() {
            summaries.push(String::new());
            continue;
        }

        let chapter_label = title
            .as_deref()
            .map(|t| format!("le chapitre \"{t}\""))
            .unwrap_or_else(|| "ce chapitre".to_string());

        let prompt = format!(
            "Résume {chapter_label} en 3 phrases maximum en français :\n\n{}",
            &text[..text.len().min(4000)]
        );

        let summary = chat_completion(client, api_key, &[("system", system), ("user", &prompt)]).await?;
        summaries.push(summary);
    }

    Ok(summaries)
}

/// Chat completion with RAG context.
pub async fn rag_chat(
    client: &reqwest::Client,
    api_key: &str,
    context: &str,
    messages: &[(String, String)],
) -> Result<String, String> {
    let system_prompt = format!(
        "Tu es un assistant bibliothécaire. Réponds aux questions en te basant sur les extraits de livres suivants. \
         Cite tes sources quand c'est pertinent. Si tu ne trouves pas la réponse dans les extraits, dis-le.\n\n\
         Extraits :\n{context}"
    );

    let mut chat_messages: Vec<(&str, &str)> = vec![("system", &system_prompt)];
    for (role, content) in messages {
        chat_messages.push((role, content));
    }

    chat_completion(client, api_key, &chat_messages).await
}

async fn chat_completion(
    client: &reqwest::Client,
    api_key: &str,
    messages: &[(&str, &str)],
) -> Result<String, String> {
    let request = ChatRequest {
        model: CHAT_MODEL.to_string(),
        messages: messages
            .iter()
            .map(|(role, content)| ChatMsg {
                role: role.to_string(),
                content: content.to_string(),
            })
            .collect(),
    };

    let resp = client
        .post(MISTRAL_CHAT_URL)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("chat request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("chat API error {status}: {body}"));
    }

    let result: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("chat parse error: {e}"))?;

    result
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "no chat response".to_string())
}
