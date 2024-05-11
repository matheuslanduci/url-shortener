use actix_web::{
    error, get,
    http::header,
    post,
    web::{Data, Json, Path},
    Error, HttpResponse,
};
use serde::Deserialize;
use validator::Validate;

use crate::{models::Url, AppState};

#[derive(Debug, Deserialize, Validate)]
struct CreateURLPayload {
    #[validate(url)]
    url: String,
    #[validate(length(min = 2, max = 12))]
    short_url: String,
    robots_allowed: bool,
}

#[post("/urls")]
pub async fn create_url(
    state: Data<AppState>,
    payload: Json<CreateURLPayload>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = payload.validate() {
        let response = HttpResponse::BadRequest().body(e.to_string());

        return Ok(response);
    }

    if payload.short_url.starts_with("/") {
        let response = HttpResponse::BadRequest().body("Short URL can't start with a slash");

        return Ok(response);
    }

    let query = r#"
        SELECT * FROM urls
        WHERE short_url = $1
    "#;

    let url = sqlx::query_as::<_, Url>(query)
        .bind(&payload.short_url)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);

            error::ErrorInternalServerError("Oops. Try again later.")
        })?;

    if url.is_some() {
        let response = HttpResponse::Conflict().body("The Short URL provided is not available");

        return Ok(response);
    }

    let query = r#"
        INSERT INTO urls (url, short_url, robots_allowed)
        VALUES ($1, $2, $3)
        RETURNING *
    "#;

    let url = sqlx::query_as::<_, Url>(query)
        .bind(&payload.url)
        .bind(&payload.short_url)
        .bind(&payload.robots_allowed)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);

            error::ErrorInternalServerError("Oops. Try again later.")
        })?;

    let response = HttpResponse::Created().json(url);

    Ok(response)
}

#[get("/urls")]
pub async fn get_urls(state: Data<AppState>) -> Result<HttpResponse, Error> {
    let query = r#"
        SELECT * FROM urls
    "#;

    let urls = sqlx::query_as::<_, Url>(query)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);

            error::ErrorInternalServerError("Oops. Try again later.")
        })?;

    let response = HttpResponse::Ok().json(urls);

    Ok(response)
}

#[get("{short_url}")]
pub async fn get_url(
    state: Data<AppState>,
    short_url: Path<String>,
) -> Result<HttpResponse, Error> {
    // Check first if the URL is in the cache
    let mut cache = state.cache.lock().unwrap();

    let short_url = short_url.into_inner();

    if let Some(url) = cache.get(&short_url) {
        let response = HttpResponse::PermanentRedirect()
            .append_header((header::LOCATION, url.url.clone()))
            .finish();

        return Ok(response);
    }

    let query = r#"
        SELECT * FROM urls WHERE short_url = $1;
    "#;

    let url = sqlx::query_as::<_, Url>(query)
        .bind(&short_url)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);

            error::ErrorInternalServerError("Oops. Try again later.")
        })?;

    cache.insert(short_url.clone(), url.clone());

    let response = HttpResponse::PermanentRedirect()
        .append_header((header::LOCATION, url.url.clone()))
        .finish();

    Ok(response)
}
