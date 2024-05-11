use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, FromRow, Serialize)]
pub struct Url {
    pub id: i32,
    pub url: String,
    pub short_url: String,
    pub robots_allowed: bool,
    pub robots_html: Option<String>,
}
