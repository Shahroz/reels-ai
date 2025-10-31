use std::num::NonZeroUsize;
use std::thread;
use tokio::sync::mpsc;
use std::path::{Path, PathBuf};
use crate::api::views::workflow::get_workflow_details::get_workflow_details;

use crate::api::views::workflow::workflow_create::create_workflow;
use crate::api::views::workflow::workflow_create::__path_create_workflow;
use crate::api::views::workflow::get_grouped_results::workflow_grouped_results;
use crate::api::views::workflow::workflow_progress::get_progress_report;
use crate::api::views::workflow::workflow_from_template::create_workflow_from_template;
use crate::api::views::workflow::get_workflow_executions::get_workflow_executions;
use crate::api::views::workflow::workflow_run::run_workflow;
use crate::api::views::node_registry::save_node_results::save_node_results;
use crate::api::views::node_registry::get_workflow_all_results::get_workflow_all_results;
use crate::api::views::node_registry::get_workflow_results::get_workflow_results;
use crate::api::views::node_registry::get_node_results::get_node_results;
use crate::api::views::workflow::workflow_from_instruction::create_workflow_from_instruction;
use crate::api::views::node_registry::get_workflows::get_workflows;
use crate::api::views::node_registry::save_node_state::save_node_state;
use crate::api::views::node_registry::get_node_state::get_node_state;
use crate::api::views::node_registry::get_content_types::get_content_types;
use crate::api::views::node_registry::pdf_to_assets::{pdf_to_assets, list_jpg_images, generate_image, analyze_pdf_art_style, generate_image_from_gcs};
use crate::api::views::node_registry::Workflow;
use crate::api::views::node_registry::get_node_results::__path_get_node_results;
use crate::api::views::node_registry::get_node_state::__path_get_node_state;
use crate::api::views::node_registry::get_workflow_all_results::__path_get_workflow_all_results;
use crate::api::views::node_registry::get_workflow_results::__path_get_workflow_results;
use crate::api::views::node_registry::get_workflows::__path_get_workflows;
use crate::api::views::node_registry::save_node_results::__path_save_node_results;
use crate::api::views::node_registry::save_node_state::__path_save_node_state;
use crate::api::views::node_registry::get_results_by_content_type::get_node_results_by_content_type;
use crate::api::views::workflow::workflow_edit::edit_workflow;
use crate::api::views::workflow::CreateWorkflowFromTemplateRequest;
use crate::api::views::workflow::NodeGraph;
use crate::api::views::workflow::WorkflowExecutorRunArguments;
use crate::api::views::workflow::workflow_from_template::__path_create_workflow_from_template;
use crate::api::views::workflow::get_workflow_details::__path_get_workflow_details;
use crate::api::views::workflow::workflow_run::__path_run_workflow;
use crate::api::views::workflow::save_actions::{icp_save_results, icp_save_workflows};
use crate::api::views::continuation_microsite::create_microsite_continuation;
use crate::api::views::node_registry::db_health::get_db_health;
use crate::api::views::ErrorResponse;
use crate::integrations::bing::BingSnippet;
use crate::nodes::nodedef::NodeDefinition;
use crate::nodes::nodedef::NodeState;
use crate::nodes::producers::scraper_node::ScrapedWebsite;
use crate::nodes::workflow_templates::WorkflowTemplateParameters;
use crate::nodes::workflows::workflow_executor::WorkflowExecutorAction;
use crate::value::NodeInnerValue;
use crate::value::NodeValue;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{available_parallelism, sleep};
use std::time::Duration;
use crate::api::views::workflow::workflow_create::CreateWorkflowRequest;

use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};

use actix_cors::Cors;
use tokio::runtime::Runtime;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use actix_web::web::{route, Data};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use awc::Client;
use dotenvy::dotenv;
use env_logger::Env;
use futures::executor;
use lazy_static::lazy_static;
use mime_guess::from_path;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::algorithms::text::string_to_array32;
use crate::api::views::smartfill::smart_fill_json;
use crate::node_registry::NodeRegistry;
use crate::nodes::context::{WorkflowContext, Message, MessageQueue, QUEUE_MSG_LIMIT};
use tokio::sync::RwLock;
use crate::nodes::output::OutputDatatype;
use crate::api::jwt_middleware::JwtMiddleware;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
#[cfg(feature = "memory")]
use crate::node_registry::memory_node_registry::MemoryNodeRegistry;
#[cfg(feature = "postgres")]
use crate::node_registry::postgresql_node_registry::PostgresSQLNodeRegistry;
#[cfg(feature = "sqlite")]
use crate::node_registry::sqlite_node_registry::SqliteNodeRegistry;
#[cfg(feature = "filebased")]
use crate::node_registry::file_node_registry::FileNodeRegistry;
#[cfg(feature = "gcs")]
use crate::node_registry::gcs_node_registry_queue::GCSNodeRegistry;

#[cfg(feature="newrelic")]
use opentelemetry::{global as otel_global, KeyValue};
#[cfg(feature="newrelic")]
use opentelemetry_sdk::runtime;
#[cfg(feature="newrelic")]
use opentelemetry_sdk::propagation::TraceContextPropagator;
#[cfg(feature="newrelic")]
use opentelemetry_sdk::resource::{EnvResourceDetector, ResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector};
#[cfg(feature="newrelic")]
use opentelemetry_sdk::trace::Config;
#[cfg(feature="newrelic")]
use actix_web_opentelemetry::RequestTracing;
use anyhow::Context;
use crate::api::rate_limiter::RateLimitMiddleware;
use crate::app::workflow_create_from_template::workflow_create_from_template;
use crate::app::workflow_run::workflow_run;
use sqlx::{PgPool, postgres::PgRow};
use crate::api::views::helpers::render_template_with_json::render_template_endpoint;
use crate::api::views::smart_table::new::smart_table_create_new;
use crate::api::views::smart_table::update::update_smart_table_endpoint;
use crate::api::views::workflow::instruction_to_gen_image_flux_shnell::instruction_to_gen_image;
use crate::api::views::workflow::instruction_to_gen_image_undraw::instruction_to_undraw_illustration;
use crate::api::views::workflow::instruction_to_gen_svg::instruction_to_svg;
use crate::api::views::workflow::instruction_to_image::instruction_to_image;
use crate::api::views::workflow::instruction_to_speech::instruction_to_speech;
use crate::api::views::workflow::instruction_to_video::instruction_to_video;
use crate::api::views::workflow::landing_page_create_new::landing_page_create_new;
use crate::api::views::workflow::landing_page_implement::landing_page_implement;
use crate::api::views::workflow::landing_page_publish::landing_page_publish;
use crate::api::views::workflow::website_style_copy_paste::style_copy_paste_endpoint;
use crate::api::views::workflow::website_style_copy_paste_ws::style_copy_paste_ws;
use crate::api::views::large_generation_ws::large_generation_ws_route;
use crate::node_registry::postgres_gcs_node_registry::PostgresSQLGCSNodeRegistry;

pub struct ProxyState {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub basic_auth_username: String,
    pub basic_auth_password: String,
    pub basic_auth_username_guest: Option<String>,
    pub basic_auth_password_guest: Option<String>,
    pub bearer_token: String,
    pub database_url: String,
    pub encryption_key: [u8; 32],
    pub gcs_bucket: Option<String>,
    pub timeout: usize,
    pub run_migrations: bool
}

impl AppConfig {
    pub fn is_local(&self) -> bool {
        self.database_url.contains("localhost")
    }
}

#[derive(RustEmbed)]
#[folder = "../ui/build/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let content_type = from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(content_type.as_ref())
                .body(content.data.into_owned())
        }
        None => handle_embedded_file("index.html"),
    }
}

#[actix_web::get("/ui/")]
async fn ui_index() -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::get("/ui/{_:.*}")]
async fn ui_dist(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}


#[derive(RustEmbed)]
#[folder = "../ui-v2/build/"]
struct AssetV2;

fn handle_embedded_file_v2(path: &str) -> HttpResponse {
    match AssetV2::get(path) {
        Some(content) => {
            let content_type = from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(content_type.as_ref())
                .body(content.data.into_owned())
        }
        None => handle_embedded_file_v2("index.html"),
    }
}

#[actix_web::get("/ui-v2/")]
async fn ui_index_v2() -> impl Responder {
    handle_embedded_file_v2("index.html")
}

#[actix_web::get("/ui-v2/{_:.*}")]
async fn ui_dist_v2(path: web::Path<String>) -> impl Responder {
    handle_embedded_file_v2(path.as_str())
}

// Health check handler
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json("Healthy")
}

async fn basic_auth_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req.app_data::<web::Data<AppConfig>>().unwrap();
    if (credentials.user_id() == config.basic_auth_username
        && credentials.password().unwrap_or_default() == config.basic_auth_password)
        || (config.basic_auth_username_guest.is_some() &&
        config.basic_auth_password_guest.is_some() &&
        credentials.user_id() == config.basic_auth_username_guest.as_ref().unwrap() &&
        credentials.password().unwrap_or_default() == config.basic_auth_password_guest.as_ref().unwrap())
        || config.is_local()
    {
        Ok(req)
    } else {
        Err((ErrorUnauthorized("Unauthorized Access"), req))
    }
}

async fn bearer_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req.app_data::<web::Data<AppConfig>>().unwrap();
    if credentials.token() == config.bearer_token {
        Ok(req)
    } else {
        Err((ErrorUnauthorized("Unauthorized Access"), req))
    }
}

#[derive(OpenApi)]
#[openapi(
paths(
create_workflow,
create_workflow_from_template,
run_workflow,
get_workflow_details,
get_node_state,
save_node_state,
get_node_results,
save_node_results,
get_workflow_results,
get_workflow_all_results,
get_workflows
),
components(schemas(
WorkflowTemplateParameters,
ScrapedWebsite,
BingSnippet,
WorkflowExecutorAction,
NodeInnerValue,
NodeDefinition,
NodeState,
NodeValue,
ErrorResponse,
Workflow,
CreateWorkflowRequest,
CreateWorkflowFromTemplateRequest,
WorkflowExecutorRunArguments,
NodeGraph,
ErrorResponse
))
)]
struct ApiDoc;

// Define a function to apply common routes to a given scope with authentication
pub fn common_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui-dummy/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    )
        .service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api/v1/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .service(get_node_state)
        .service(save_node_state)
        .service(get_workflows)
        .service(smart_fill_json)
        .service(get_workflow_details)
        .service(create_workflow_from_instruction)
        .service(get_node_results)
        .service(get_workflow_results)
        .service(get_workflow_all_results)
        .service(save_node_results)
        .service(run_workflow)
        .service(get_workflow_executions)
        .service(create_workflow_from_template)
        .service(get_progress_report)
        .service(workflow_grouped_results)
        .service(create_microsite_continuation)
        .service(edit_workflow)
        .service(get_node_results_by_content_type)
        .service(get_content_types)
        .service(get_db_health)
        .service(create_workflow)
        .service(icp_save_results)
        .service(render_template_endpoint)
        .service(icp_save_workflows)
        .service(smart_table_create_new)
        .service(update_smart_table_endpoint)
        .service(landing_page_create_new)
        .service(landing_page_implement)
        .service(list_jpg_images)
        .service(generate_image)
        .service(style_copy_paste_endpoint)
        .service(landing_page_publish);
}

pub fn public_api_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(create_workflow_from_template)
        .service(get_node_results_by_content_type)
        .service(get_workflow_executions)
        .service(get_progress_report)
        .service(run_workflow)
        .service(icp_save_results)
        .service(icp_save_workflows)
        .service(list_jpg_images)
        .service(generate_image)
        .service(pdf_to_assets)
        .service(analyze_pdf_art_style)
        .service(generate_image_from_gcs);
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeRegistryImplementation {
    #[cfg(feature = "postgres")]
    PostgreSQL,
    #[cfg(feature = "sqlite")]
    SQLite,
}

#[cfg(feature = "sqlite")]
pub async fn create_sqlite_db() -> Result<(), sqlx::Error> {
    let database_path = "db.sqlite";

    // Create the SQLite connection options
    let mut connect_options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true);

    // Create a new SQLite connection pool
    let pool = SqlitePool::connect_with(connect_options).await?;

    // Execute the PRAGMA statement to enable WAL mode
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await?;

    log::debug!(
        "Database created at {} with WAL mode enabled",
        database_path
    );

    Ok(())
}

const DEFAULT_TIMEOUT: &'static str = "1200";

pub struct StopperSignal;

pub async fn run_app() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("html5ever", log::LevelFilter::Off)
        .filter_module("html2text", log::LevelFilter::Off)
        .init();

    // try to load .env file but it if fails that's ok the env variables in production are
    // set globally
    _ = dotenv();

    // Start configuring a `fmt` subscriber
    // let subscriber = tracing_subscriber::fmt()
    //     .json()
    //     .with_current_span(false)
    //     .flatten_event(true)
    //     .with_span_list(false)
    //     .with_target(false)
    //     .with_file(false)
    //     .with_level(true)
    //     .init();

    #[cfg(feature="newrelic")]
    otel_global::set_text_map_propagator(TraceContextPropagator::new());
    #[cfg(feature="newrelic")]
        let sdk_provided_resource = SdkProvidedResourceDetector.detect(Duration::from_secs(0));
    #[cfg(feature="newrelic")]
        let env_resource = EnvResourceDetector::new().detect(Duration::from_secs(0));
    #[cfg(feature="newrelic")]
        let telemetry_resource = TelemetryResourceDetector.detect(Duration::from_secs(0));
    #[cfg(feature="newrelic")]
        let resource = sdk_provided_resource
        .merge(&env_resource)
        .merge(&telemetry_resource);
    #[cfg(feature="newrelic")]
        let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic()
            .with_tls_config(tonic::transport::ClientTlsConfig::new().with_native_roots()))
        .with_trace_config(Config::default().with_resource(resource))
        .install_batch(runtime::Tokio)
        .expect("failed to initialize the trace pipeline");
    #[cfg(feature="newrelic")]
    otel_global::set_tracer_provider(tracer_provider);

    let app_config = init_app_config();

    let port_string = std::env::var("PORT").expect("PORT must be set");
    let port = u16::from_str(&port_string).expect("cannot parse port as integer");

    let node_registry = init_node_registry(&app_config).await;

    log::info!("starting HTTP server at http://0.0.0.0:{}", port);

    let node_context = init_node_context(node_registry, &app_config);

    let reqwest_client = reqwest::Client::default();

    let openapi = ApiDoc::openapi();

    let parallelism = available_parallelism().expect("Cannot extract the number of CPUs").get();
    log::info!("available parallelism: {}", parallelism);
    let n_workers = parallelism / 2;

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|_origin, _req_head| true)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        let basic_auth = HttpAuthentication::basic(basic_auth_validator);
        let bearer_auth = HttpAuthentication::bearer(bearer_auth_validator);

        let mut app = App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(reqwest_client.clone()))
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(node_context.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(ui_index)
            .service(ui_dist)
            .service(ui_index_v2)
            .service(ui_dist_v2)
            .service(instruction_to_image)
            .service(instruction_to_video)
            .service(instruction_to_gen_image)
            .service(instruction_to_speech)
            .service(instruction_to_undraw_illustration)
            .service(instruction_to_svg)
            .route("/ws/style-copy-paste", web::get().to(style_copy_paste_ws))
            .route("/ws/large-generation", web::get().to(large_generation_ws_route))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .wrap(basic_auth.clone())
                    .configure(common_api_routes),
            )
            .service(
                web::scope("/api/v2")
                    .wrap(bearer_auth.clone())
                    .configure(common_api_routes),
            ).service(
            // public
            web::scope("/api/public")
                // .wrap(RateLimitMiddleware::new(node_registry.get_pool(), 1000, Duration::from_secs(60)))
                .configure(public_api_routes))
            .service(
                // Separate service for leadtalk related endpoint, with JWT authentication
                web::scope("/api/jwt").wrap(JwtMiddleware)
                    .configure(common_api_routes),
        );

        #[cfg(feature = "newrelic")]
        {
            app.wrap(RequestTracing::new())
        }

        #[cfg(not(feature = "newrelic"))]
        {
            app
        }
    })
        .bind(("0.0.0.0", port))?
        .workers(n_workers)
        // prevent 408 errors?
        .client_request_timeout(Duration::from_secs(30))
        .run();

    server.await;

    #[cfg(feature="newrelic")]
    otel_global::shutdown_tracer_provider();

    Ok(())
}

//Node context in this form is probably too outdates - the datatypes shouldn't be defined here
pub fn init_node_context(node_registry: Arc<dyn NodeRegistry>, app_config: &AppConfig) -> WorkflowContext {
    let (tx, rx) = mpsc::channel::<Message>(QUEUE_MSG_LIMIT);
    let node_context = WorkflowContext {
        datatypes: vec![
            OutputDatatype::Reference,
            OutputDatatype::Email,
            OutputDatatype::UseCaseV1,
            OutputDatatype::Contact,
            OutputDatatype::Pitch,
        ],
        timeout: app_config.timeout,
        node_registry: Some(node_registry.clone()),
        node_n_results: Arc::new(Default::default()),
        execution_id: None,
        tenant_id: None,
        message_queue: Arc::new(MessageQueue {
            sender: tx,
            receiver: Arc::new(RwLock::new(rx)),
        }),
    };
    node_context
}

pub fn init_app_config() -> AppConfig {
    let timeout = std::env::var("DEFAULT_TIMEOUT").unwrap_or(DEFAULT_TIMEOUT.to_string());
    let timeout_num = usize::from_str(&timeout).expect("DEFAULT_TIMEOUT must be a number");
    let app_config = AppConfig {
        basic_auth_username: std::env::var("UI_USERNAME").expect("UI_USERNAME must be set"),
        basic_auth_password: std::env::var("UI_PASSWORD").expect("UI_PASSWORD must be set"),
        basic_auth_username_guest: std::env::var("UI_USERNAME_GUEST").ok(),
        basic_auth_password_guest: std::env::var("UI_PASSWORD_GUEST").ok(),
        bearer_token: std::env::var("BEARER_API_TOKEN").expect("BEARER_API_TOKEN must be set"),
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        gcs_bucket: std::env::var("GCS_BUCKET").ok(),
        encryption_key: string_to_array32(
            std::env::var("ENCRYPTION_KEY")
                .expect("ENCRYPTION_KEY must be set")
                .as_str(),
        ).expect("ENCRYPTION_KEY must be exactly 32 characters"),
        timeout: timeout_num,
        run_migrations: std::env::var("RUN_MIGRATIONS").unwrap_or("1".to_string())=="1"
    };
    app_config
}

pub async fn init_node_registry(app_config: &AppConfig) -> Arc<dyn NodeRegistry> {
    let node_registry: Arc<dyn NodeRegistry> = {
        #[cfg(feature = "gcs")]
        {
            Arc::new(GCSNodeRegistry::new(app_config.gcs_bucket.clone().expect("Cannot run gcs feature withouth GCS_BUCKET env var")))
        }
        #[cfg(feature = "filebased")]
        {
            Arc::new(FileNodeRegistry::new(PathBuf::from("node_registry")))
        }
        #[cfg(feature = "memory")]
        {
            Arc::new(MemoryNodeRegistry::new())
        }
        #[cfg(feature = "postgres")]
        {
            let database_url = app_config.database_url.clone();

            let min_db_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|val| val.parse::<u32>().ok())
                .unwrap_or(10);

            let max_db_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|val| val.parse::<u32>().ok())
                .unwrap_or(400);

            let pool = sqlx::postgres::PgPoolOptions::new()
                .acquire_timeout(Duration::from_secs(120))
                .idle_timeout(Some(Duration::from_secs(60)))
                .min_connections(min_db_connections)
                .max_connections(max_db_connections)
                .connect(&database_url)
                .await
                .expect("Failed to connect to the database");

            // Don't run migrations here
            if app_config.run_migrations {
                sqlx::migrate!("./../migrations")
                    .run(&pool)
                    .await
                    .expect("Could not run migrations");
            }

            let bucket_name = std::env::var("NODE_REGISTRY_BUCKET_NAME").unwrap_or("gennodes_node_registry".to_string());
            Arc::new(PostgresSQLGCSNodeRegistry::new(pool.clone(), bucket_name).await.expect("Cannot initialize PostgreSQLGCS Node registry"))
        }

        #[cfg(feature = "sqlite")]
        {
            create_sqlite_db().await;
            let mut options: SqliteConnectOptions = SqliteConnectOptions::new();
            options = options.filename("db.sqlite");

            let pool = SqlitePool::connect_with(options.clone())
                .await
                .expect("Failed to connect to the SQLite database");

            if app_config.run_migrations {
                sqlx::migrate!("./sqlite_migrations")
                    .run(&pool)
                    .await
                    .expect("Could not run migrations");
            }

            Arc::new(SqliteNodeRegistry::new(pool.clone()))
        }
    };
    node_registry
}