use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Feature {
    name: String,
    status: String, // "complete", "incomplete", "staged"
    priority: String, // "critical", "high", "medium", "low"
    lines_changed: usize,
    files_affected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bug {
    id: String,
    severity: String,
    description: String,
    file_path: String,
    line_number: usize,
    status: String, // "open", "fixed", "wontfix"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiffChange {
    file_path: String,
    change_type: String, // "added", "modified", "deleted"
    lines_added: usize,
    lines_removed: usize,
    complexity_delta: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisResult {
    project_name: String,
    timestamp: String,
    total_files: usize,
    total_violations: usize,
    violations_by_severity: HashMap<String, usize>,
    files_with_violations: usize,
    
    // Enhanced metrics
    drift_percentage: f64,
    drift_count: usize,
    
    // Feature tracking
    features_shipped: HashMap<String, usize>, // major, minor, patch
    incomplete_features: Vec<Feature>,
    
    // Bug tracking
    bugs_found: Vec<Bug>,
    bugs_by_severity: HashMap<String, usize>,
    
    // Diff visualization
    diff_changes: Vec<DiffChange>,
    
    // Hierarchical insights
    insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Insight {
    category: String,
    priority: String,
    title: String,
    description: String,
    actionable: bool,
    affected_files: Vec<String>,
}

struct AppState {
    results: Arc<Mutex<Vec<AnalysisResult>>>,
    tera: Tera,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let tera = Tera::new("templates/**/*")?;
    
    let state = web::Data::new(AppState {
        results: Arc::new(Mutex::new(Vec::new())),
        tera,
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_state(state.clone())
            .wrap(cors)
            .route("/", web::get().to(dashboard))
            .route("/api/results", web::get().to(get_results))
            .route("/api/results", web::post().to(add_result))
            .route("/api/results/clear", web::post().to(clear_results))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

async fn dashboard(state: web::Data<AppState>) -> impl Responder {
    let results = state.results.lock().unwrap();
    
    // Calculate aggregate statistics
    let total_violations: usize = results.iter().map(|r| r.total_violations).sum();
    let total_files: usize = results.iter().map(|r| r.total_files).sum();
    let avg_drift: f64 = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| r.drift_percentage).sum::<f64>() / results.len() as f64
    };
    
    let total_bugs: usize = results.iter().map(|r| r.bugs_found.len()).sum();
    let total_incomplete: usize = results.iter().map(|r| r.incomplete_features.len()).sum();
    
    // Aggregate severity breakdown
    let mut aggregated_severity: HashMap<String, usize> = HashMap::new();
    for result in &*results {
        for (severity, count) in &result.violations_by_severity {
            *aggregated_severity.entry(severity.clone()).or_insert(0) += count;
        }
    }
    
    // Aggregate features shipped
    let mut aggregated_features: HashMap<String, usize> = HashMap::new();
    for result in &*results {
        for (feature_type, count) in &result.features_shipped {
            *aggregated_features.entry(feature_type.clone()).or_insert(0) += count;
        }
    }
    
    // Collect all insights grouped by priority
    let mut insights_by_priority: HashMap<String, Vec<Insight>> = HashMap::new();
    for result in &*results {
        for insight in &result.insights {
            insights_by_priority
                .entry(insight.priority.clone())
                .or_insert_with(Vec::new)
                .push(insight.clone());
        }
    }
    
    let mut context = Context::new();
    context.insert("results", &*results);
    context.insert("total_projects", &results.len());
    context.insert("total_violations", &total_violations);
    context.insert("total_files", &total_files);
    context.insert("avg_drift", &avg_drift);
    context.insert("total_bugs", &total_bugs);
    context.insert("total_incomplete", &total_incomplete);
    context.insert("aggregated_severity", &aggregated_severity);
    context.insert("aggregated_features", &aggregated_features);
    context.insert("insights_by_priority", &insights_by_priority);
    
    let html = state.tera.render("dashboard.html", &context).unwrap_or_else(|e| {
        format!("Error rendering template: {}", e)
    });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

async fn get_results(state: web::Data<AppState>) -> impl Responder {
    let results = state.results.lock().unwrap();
    HttpResponse::Ok().json(&*results)
}

async fn add_result(
    result: web::Json<AnalysisResult>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut results = state.results.lock().unwrap();
    results.push(result.into_inner());
    HttpResponse::Ok().json("Result added")
}

async fn clear_results(state: web::Data<AppState>) -> impl Responder {
    let mut results = state.results.lock().unwrap();
    results.clear();
    HttpResponse::Ok().json("Results cleared")
}
