/// Axum handler for sitemap.xml
pub async fn sitemap_handler(
    axum::extract::State(state): axum::extract::State<super::state::AppState>,
) -> impl axum::response::IntoResponse {
    let books = super::db::list_all_book_references(&state.pool)
        .await
        .unwrap_or_default();

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://bilbo.example.com/</loc>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://bilbo.example.com/chat</loc>
    <priority>0.5</priority>
  </url>
"#,
    );

    for (reference, _title) in &books {
        xml.push_str(&format!(
            "  <url>\n    <loc>https://bilbo.example.com/book/{reference}</loc>\n    <priority>0.8</priority>\n  </url>\n"
        ));
    }

    xml.push_str("</urlset>");

    (
        [(http::header::CONTENT_TYPE, "application/xml")],
        xml,
    )
}
