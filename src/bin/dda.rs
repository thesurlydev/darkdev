use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
    response::{IntoResponse, Response},
    extract::State
};
use axum::serve;
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone, Serialize)]
struct ApiError {
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    name: String,
    path: String,
    mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitProject {
    name: String,
    project_type: String,
}

#[derive(Debug, Serialize)]
struct HealthCheck {
    status: String,
    version: String,
}

#[derive(Debug, Clone)]
struct AppState {
    projects: Arc<Mutex<Vec<Project>>>,
}

async fn health_check() -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn init_project(
    State(_state): State<Arc<AppState>>,
    Json(project): Json<InitProject>,
) -> Result<Json<Project>, ApiError> {
    // TODO: Implement actual project initialization
    let name = project.name.clone();
    log::info!("Initializing new {} project: {}", project.project_type, name);
    
    Ok(Json(Project {
        name: name.clone(),
        path: format!("./{}", name),
        mode: "compile".to_string(),
    }))
}

async fn add_project(
    State(state): State<Arc<AppState>>,
    Json(project): Json<Project>,
) -> Result<Json<Project>, ApiError> {
    let mut projects = state.projects.lock().await;
    if projects.iter().any(|p| p.name == project.name) {
        return Err(ApiError {
            message: format!("Project {} already exists", project.name),
        });
    }
    projects.push(project.clone());
    Ok(Json(project))
}

async fn remove_project(
    State(state): State<Arc<AppState>>,
    Json(project): Json<Project>,
) -> Result<StatusCode, ApiError> {
    let mut projects = state.projects.lock().await;
    if let Some(pos) = projects.iter().position(|p| p.name == project.name) {
        projects.remove(pos);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError {
            message: format!("Project {} not found", project.name),
        })
    }
}

async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Project>> {
    let projects = state.projects.lock().await;
    Json(projects.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let state = Arc::new(AppState {
        projects: Arc::new(Mutex::new(Vec::new())),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/projects/init", post(init_project))
        .route("/projects", post(add_project))
        .route("/projects", get(list_projects))
        .route("/projects/remove", post(remove_project))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    log::info!("Listening on {}", listener.local_addr()?);    
    serve::serve(listener, app).await?;
    Ok(())
}