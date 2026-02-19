use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, Filter, PointStruct,
    ScalarQuantizationBuilder, VectorParamsBuilder,
    CreateFieldIndexCollectionBuilder, FieldType,
    SearchPointsBuilder, UpsertPointsBuilder, DeletePointsBuilder,
    Condition, PointId, points_selector::PointsSelectorOneOf,
};
use qdrant_client::Qdrant;
use serde_json::json;
use uuid::Uuid;

pub const COLLECTION_NAME: &str = "book_chunks";
const VECTOR_SIZE: u64 = 1024;

pub async fn ensure_collection(client: &Qdrant) -> Result<(), qdrant_client::QdrantError> {
    let exists = client.collection_exists(COLLECTION_NAME).await?;
    if exists {
        return Ok(());
    }

    client
        .create_collection(
            CreateCollectionBuilder::new(COLLECTION_NAME)
                .vectors_config(VectorParamsBuilder::new(VECTOR_SIZE, Distance::Cosine))
                .quantization_config(ScalarQuantizationBuilder::default()),
        )
        .await?;

    client
        .create_field_index(
            CreateFieldIndexCollectionBuilder::new(COLLECTION_NAME, "book_id", FieldType::Keyword),
        )
        .await?;
    client
        .create_field_index(
            CreateFieldIndexCollectionBuilder::new(COLLECTION_NAME, "tags", FieldType::Keyword),
        )
        .await?;
    client
        .create_field_index(
            CreateFieldIndexCollectionBuilder::new(COLLECTION_NAME, "authors", FieldType::Keyword),
        )
        .await?;

    Ok(())
}

pub async fn delete_book_points(
    client: &Qdrant,
    book_id: Uuid,
) -> Result<(), qdrant_client::QdrantError> {
    let filter = Filter::must([Condition::matches(
        "book_id",
        book_id.to_string(),
    )]);

    client
        .delete_points(
            DeletePointsBuilder::new(COLLECTION_NAME)
                .points(PointsSelectorOneOf::Filter(filter)),
        )
        .await?;

    Ok(())
}

pub async fn upsert_chunks(
    client: &Qdrant,
    book_id: Uuid,
    reference: &str,
    title: &str,
    authors: &[String],
    tags: &[String],
    chunks: &[(usize, Option<String>, usize, String)],
    embeddings: &[Vec<f32>],
) -> Result<(), qdrant_client::QdrantError> {
    let mut points = Vec::new();

    for ((chapter_idx, chapter_title, chunk_index, text), embedding) in
        chunks.iter().zip(embeddings.iter())
    {
        let point_id = Uuid::new_v4();
        let payload = json!({
            "book_id": book_id.to_string(),
            "reference": reference,
            "title": title,
            "chunk_index": chunk_index,
            "chunk_text": text,
            "chapter_idx": chapter_idx,
            "chapter": chapter_title.as_deref().unwrap_or(""),
            "authors": authors,
            "tags": tags,
        });

        let payload_map: std::collections::HashMap<String, qdrant_client::qdrant::Value> =
            serde_json::from_value(payload).unwrap_or_default();

        points.push(PointStruct::new(
            PointId::from(point_id.to_string()),
            embedding.clone(),
            payload_map,
        ));

        if points.len() >= 100 {
            let batch: Vec<PointStruct> = points.drain(..).collect();
            client
                .upsert_points(
                    UpsertPointsBuilder::new(COLLECTION_NAME, batch),
                )
                .await?;
        }
    }

    if !points.is_empty() {
        client
            .upsert_points(
                UpsertPointsBuilder::new(COLLECTION_NAME, points),
            )
            .await?;
    }

    Ok(())
}

pub struct SearchResult {
    pub reference: String,
    pub title: String,
    pub chunk_text: String,
    pub score: f32,
}

pub async fn search_similar(
    client: &Qdrant,
    query_embedding: Vec<f32>,
    tags: &[String],
    author: Option<&str>,
    limit: u64,
) -> Result<Vec<SearchResult>, qdrant_client::QdrantError> {
    let mut search = SearchPointsBuilder::new(COLLECTION_NAME, query_embedding, limit)
        .with_payload(true);

    let mut conditions: Vec<Condition> = tags
        .iter()
        .map(|t| Condition::matches("tags", t.clone()))
        .collect();

    if let Some(a) = author {
        conditions.push(Condition::matches("authors", a.to_string()));
    }

    if !conditions.is_empty() {
        search = search.filter(Filter::must(conditions));
    }

    let results = client.search_points(search).await?;

    let mut search_results = Vec::new();
    for point in results.result {
        let payload = &point.payload;
        let reference = payload
            .get("reference")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let title = payload
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let chunk_text = payload
            .get("chunk_text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        search_results.push(SearchResult {
            reference,
            title,
            chunk_text,
            score: point.score,
        });
    }

    Ok(search_results)
}
