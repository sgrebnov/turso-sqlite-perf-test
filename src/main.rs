use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{duckdb::DuckDbConnectionPool, sqlite::SqliteConnectionPool};

mod duckdb;
mod sqlite;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: Option<String>,
}

async fn sql_duckdb(State((duckdb_pool, _sqlite_pool)): State<(Arc<DuckDbConnectionPool>, Arc<SqliteConnectionPool>)>, body: String) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
   let sql_query = body.trim();
    
    if sql_query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Empty SQL query".to_string(),
                message: None,
            }),
        ));
    }

    match duckdb::execute_query(&duckdb_pool, sql_query) {
        Ok(result) => {
            if result.len() != 1 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Expected exactly one row".to_string(),
                        message: None,
                    }),
                ));
            }
            let res = serde_json::json!({
                "result": format!("{:?}", result)
            });
            Ok(Json(res))
        }
        Err(e) => {
            println!("DuckDB query execution error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DuckDB query execution failed".to_string(),
                    message: Some(e.to_string()),
                }),
            ))
        }
    }
}


async fn sql_turso(State((_duckdb_pool, sqlite_pool)): State<(Arc<DuckDbConnectionPool>, Arc<SqliteConnectionPool>)>, body: String) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let sql_query = body.trim();

    if sql_query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Empty SQL query".to_string(),
                message: None,
            }),
        ));
    }

    match sqlite::execute_query(&sqlite_pool, sql_query).await {
        Ok(result) => {
            if result.len() != 1 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Expected exactly one row".to_string(),
                        message: None,
                    }),
                ));
            }
            let res = serde_json::json!({
                "result": format!("{:?}", result)
            });
            Ok(Json(res))
        }
        Err(e) => {
            println!("SQLite query execution error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SQLite query execution failed".to_string(),
                    message: Some(e.to_string()),
                }),
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    let current_dir = std::env::current_dir()
        .expect("Failed to get current directory");
    let current_dir_str = current_dir
        .to_str()
        .expect("Failed to convert path to string");

    let duckdb_pool = Arc::new(
        DuckDbConnectionPool::new(format!("{current_dir_str}/duckdb_test.db").as_str())
            .expect("Failed to create DuckDB connection pool")
    );
    
    let sqlite_pool = Arc::new(
        SqliteConnectionPool::new(format!("{current_dir_str}/sqlite_test.db").as_str())
            .await
            .expect("Failed to create SQLite connection pool")
    );

    let app = Router::new()
        .route("/v1/sql_duckdb", post(sql_duckdb))
        .route("/v1/sql_turso", post(sql_turso))
        .with_state((duckdb_pool.clone(), sqlite_pool.clone()))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8090")
        .await
        .expect("To bind TCP listener");

    println!("SQL query server running at http://localhost:8090");
    println!("SQL endpoint 1: POST http://localhost:8090/v1/sql_duckdb");
    println!("SQL endpoint 2: POST http://localhost:8090/v1/sql_turso");

    axum::serve(listener, app).await.expect("To start server successfully");
}

