use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Decimal;
use sqlx::{PgPool, Row, Column};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct QueryResult {
    data: Vec<serde_json::Value>,
    count: usize,
    page: usize,
    page_size: usize,
    total_count: Option<usize>,
}

#[derive(Debug)]
struct FilterCondition {
    column: String,
    operator: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
    sort: Option<String>,
    order: Option<String>,
}

fn parse_filter(filter_str: &str) -> Result<FilterCondition, String> {
    // URL decode the filter string
    let decoded = urlencoding::decode(filter_str)
        .map_err(|_| "Failed to decode URL".to_string())?
        .to_string();
    
    let operators = vec![">=", "<=", "!=", "=", ">", "<"];
    
    for op in operators {
        if let Some(pos) = decoded.find(op) {
            let column = decoded[..pos].trim().to_string();
            let value = decoded[pos + op.len()..].trim().to_string();
            
            if column.is_empty() || value.is_empty() {
                return Err("Invalid filter format".to_string());
            }
            
            return Ok(FilterCondition {
                column,
                operator: op.to_string(),
                value,
            });
        }
    }
    
    Err("No valid operator found".to_string())
}

fn parse_multiple_filters(filters_str: &str) -> Result<Vec<FilterCondition>, String> {
    let decoded = urlencoding::decode(filters_str)
        .map_err(|_| "Failed to decode URL".to_string())?
        .to_string();
    
    let filter_parts: Vec<&str> = decoded.split('&').collect();
    let mut conditions = Vec::new();
    
    for part in filter_parts {
        let filter = parse_filter(part)?;
        conditions.push(filter);
    }
    
    Ok(conditions)
}

fn sanitize_table_name(table: &str) -> Result<String, String> {
    if !table.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Invalid table name".to_string());
    }
    Ok(table.to_string())
}

fn sanitize_column_name(column: &str) -> Result<String, String> {
    if !column.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Invalid column name".to_string());
    }
    Ok(column.to_string())
}

fn validate_sort_order(order: &str) -> Result<String, String> {
    let order_upper = order.to_uppercase();
    if order_upper == "ASC" || order_upper == "DESC" {
        Ok(order_upper)
    } else {
        Err("Invalid sort order. Use 'asc' or 'desc'".to_string())
    }
}

async fn query_table(
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
    query_params: web::Query<QueryParams>,
) -> impl Responder {
    let (table_name, filters_str) = path.into_inner();
    
    // Sanitize table name
    let table = match sanitize_table_name(&table_name) {
        Ok(t) => t,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    };
    
    // Parse filters
    let filters = match parse_multiple_filters(&filters_str) {
        Ok(f) => f,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Invalid filter: {}", e)
        })),
    };
    
    // Validate and sanitize all column names
    let mut sanitized_filters = Vec::new();
    for filter in filters {
        let column = match sanitize_column_name(&filter.column) {
            Ok(c) => c,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            })),
        };
        sanitized_filters.push(FilterCondition {
            column,
            operator: filter.operator,
            value: filter.value,
        });
    }
    
    // Pagination parameters
    let page = query_params.page.unwrap_or(1);
    let page_size = query_params.page_size.unwrap_or(100).min(1000); // Max 1000 per page
    let offset = (page - 1) * page_size;
    
    // Sort parameters
    let sort_column = if let Some(ref sort) = query_params.sort {
        match sanitize_column_name(sort) {
            Ok(c) => Some(c),
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            })),
        }
    } else {
        None
    };
    
    let sort_order = if let Some(ref order) = query_params.order {
        match validate_sort_order(order) {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            })),
        }
    } else {
        "ASC".to_string()
    };
    
    // Build WHERE clause with proper type casting
    let where_clause = sanitized_filters
        .iter()
        .enumerate()
        .map(|(i, f)| {
            // Try to detect the type and cast accordingly
            // For date comparisons, cast the parameter to date
            format!("{}::text {} ${}::text", f.column, f.operator, i + 1)
        })
        .collect::<Vec<String>>()
        .join(" AND ");
    
    // Build ORDER BY clause
    let order_by_clause = if let Some(col) = &sort_column {
        format!(" ORDER BY {} {}", col, sort_order)
    } else {
        String::new()
    };
    
    // Count query for pagination
    let count_query = format!(
        "SELECT COUNT(*) as count FROM {} WHERE {}",
        table, where_clause
    );
    
    // Main query with pagination
    let query = format!(
        "SELECT * FROM {} WHERE {}{} LIMIT {} OFFSET {}",
        table, where_clause, order_by_clause, page_size, offset
    );
    
    log::info!("Executing query: {}", query);
    log::info!("With values: {:?}", sanitized_filters.iter().map(|f| &f.value).collect::<Vec<_>>());
    
    // Get total count
    let mut count_query_builder = sqlx::query(&count_query);
    for filter in &sanitized_filters {
        count_query_builder = count_query_builder.bind(&filter.value);
    }
    
    let total_count = match count_query_builder.fetch_one(pool.get_ref()).await {
        Ok(row) => {
            let count: i64 = row.try_get("count").unwrap_or(0);
            Some(count as usize)
        }
        Err(e) => {
            log::error!("Count query error: {}", e);
            None
        }
    };
    
    // Execute main query
    let mut query_builder = sqlx::query(&query);
    for filter in &sanitized_filters {
        query_builder = query_builder.bind(&filter.value);
    }
    
    match query_builder.fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let mut results = Vec::new();
            
            for row in &rows {
                let mut obj = serde_json::Map::new();
                
                for (i, column) in row.columns().iter().enumerate() {
                    let col_name = column.name();
                    
                    // Try to get value as different types
                    let value: serde_json::Value = if let Ok(v) = row.try_get::<i32, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<i64, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<f64, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<f32, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<Decimal, _>(i) {
                        serde_json::json!(v.to_string())
                    } else if let Ok(v) = row.try_get::<String, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<bool, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                        serde_json::json!(v.to_string())
                    } else if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(i) {
                        serde_json::json!(v.to_string())
                    } else {
                        serde_json::json!(null)
                    };
                    
                    obj.insert(col_name.to_string(), value);
                }
                
                results.push(serde_json::Value::Object(obj));
            }
            
            let response = QueryResult {
                count: results.len(),
                data: results,
                page,
                page_size,
                total_count,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

async fn query_all(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    query_params: web::Query<QueryParams>,
) -> impl Responder {
    let table_name = path.into_inner();
    
    // Sanitize table name
    let table = match sanitize_table_name(&table_name) {
        Ok(t) => t,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    };
    
    // Pagination parameters
    let page = query_params.page.unwrap_or(1);
    let page_size = query_params.page_size.unwrap_or(100).min(1000);
    let offset = (page - 1) * page_size;
    
    // Sort parameters
    let sort_column = if let Some(ref sort) = query_params.sort {
        match sanitize_column_name(sort) {
            Ok(c) => Some(c),
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            })),
        }
    } else {
        None
    };
    
    let sort_order = if let Some(ref order) = query_params.order {
        match validate_sort_order(order) {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            })),
        }
    } else {
        "ASC".to_string()
    };
    
    let order_by_clause = if let Some(col) = &sort_column {
        format!(" ORDER BY {} {}", col, sort_order)
    } else {
        String::new()
    };
    
    // Count query
    let count_query = format!("SELECT COUNT(*) as count FROM {}", table);
    
    // Main query
    let query = format!(
        "SELECT * FROM {}{} LIMIT {} OFFSET {}",
        table, order_by_clause, page_size, offset
    );
    
    log::info!("Executing query: {}", query);
    
    // Get total count
    let total_count = match sqlx::query(&count_query).fetch_one(pool.get_ref()).await {
        Ok(row) => {
            let count: i64 = row.try_get("count").unwrap_or(0);
            Some(count as usize)
        }
        Err(e) => {
            log::error!("Count query error: {}", e);
            None
        }
    };
    
    // Execute main query
    match sqlx::query(&query).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let mut results = Vec::new();
            
            for row in &rows {
                let mut obj = serde_json::Map::new();
                
                for (i, column) in row.columns().iter().enumerate() {
                    let col_name = column.name();
                    
                    let value: serde_json::Value = if let Ok(v) = row.try_get::<i32, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<i64, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<f64, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<f32, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<Decimal, _>(i) {
                        serde_json::json!(v.to_string())
                    } else if let Ok(v) = row.try_get::<String, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<bool, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                        serde_json::json!(v.to_string())
                    } else if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(i) {
                        serde_json::json!(v.to_string())
                    } else {
                        serde_json::json!(null)
                    };
                    
                    obj.insert(col_name.to_string(), value);
                }
                
                results.push(serde_json::Value::Object(obj));
            }
            
            let response = QueryResult {
                count: results.len(),
                data: results,
                page,
                page_size,
                total_count,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    log::info!("Connected to database");
    
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);
    
    log::info!("Starting server at {}", bind_address);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health_check))
            .route("/{table}", web::get().to(query_all))
            .route("/{table}/{filter}", web::get().to(query_table))
    })
    .bind(&bind_address)?
    .run()
    .await
}