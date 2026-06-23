use std::sync::Arc;
use crate::endpoint::ratelimit::IpKeyExtractor;
use crate::model::config::AppConfig;
use crate::model::keys::Keys;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{get, web, App, HttpServer, Responder};
use anyhow::Context;
use fern::Dispatch;
use log::LevelFilter;
use utoipa::openapi::InfoBuilder;
use utoipa_actix_web::{scope, AppExt};
use utoipa_swagger_ui::SwaggerUi;
use crate::database::postgres::PostgresDatabase;
use crate::model::database::Database;

pub mod model;
pub mod endpoint;
pub mod database;

pub struct AppState {
    pub keys: Keys,
    pub database: Arc<dyn Database>,
}

#[utoipa::path(summary = "Index", responses(
    (status = 200, description = "API is running")
))]
#[get("/")]
async fn index() -> impl Responder {
    "API is running. Visit /swagger-ui/ for documentation."
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()
        .context("Failed to load .env file")?;
    setup_logger()
        .context("Failed to initialize logger")?;

    let config = AppConfig::from_env();

    let config_clone = config.clone();

    let governor_conf = GovernorConfigBuilder::default()
        .requests_per_minute(60)
        .key_extractor(IpKeyExtractor)
        .finish()
        .context("Failed to create governor config")?;

    let governor_daily_conf = GovernorConfigBuilder::default()
        .seconds_per_request(86400 / 5000)
        .burst_size(5000)
        .key_extractor(IpKeyExtractor)
        .finish()
        .context("Failed to create daily governor config")?;

    let database = Arc::new(PostgresDatabase::new(config.database_url.as_str(), 10).await
        .context("Failed to connect to Postgres database")?
    );

    let app_state = web::Data::new(AppState {
        keys: Keys::from_master_key(config.master_key.as_str()),
        database,
    });

    HttpServer::new(move || {
        let (app, _) = App::new()
            .wrap(Governor::new(&governor_conf))
            .wrap(Governor::new(&governor_daily_conf))
            .into_utoipa_app()
            .app_data(app_state.clone())
            .app_data(web::Data::new(config.clone()))
            .service(index)
            .service(scope::scope("/api/v1")
                .service(scope::scope("/protected")
                    .service(endpoint::protected::protected)
                )
                .service(scope::scope("/auth")
                    .service(endpoint::auth::create)
                )
            )
            .openapi_service(|mut api| {
                api.info = InfoBuilder::new()
                    .title(env!("CARGO_PKG_NAME").to_string())
                    .version(env!("CARGO_PKG_VERSION").to_string())
                    .build();

                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .split_for_parts();
        app
    })
        .workers(4)
        .max_connections(200)
        .bind((config_clone.server_address, config_clone.server_port))
        .context("Failed to bind server")?
        .run()
        .await
        .context("Server error")
}

fn setup_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}