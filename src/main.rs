use std::{collections::HashMap, sync::Mutex};

use actix_web::{
    middleware::Logger,
    web::{self, Data, ServiceConfig},
};
use models::Url;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{migrate, PgPool};

mod models;
mod routes;

pub struct AppState {
    pool: PgPool,
    cache: Mutex<HashMap<String, Url>>,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations. Check your database.");

    let cache = Mutex::new(HashMap::new());

    let state = Data::new(AppState { pool, cache });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("")
                .wrap(Logger::default())
                .service(routes::url_routes::get_urls)
                .service(routes::url_routes::get_url)
                //.service(routes::url_routes::create_url)
                .app_data(state),
        );
    };

    Ok(config.into())
}
