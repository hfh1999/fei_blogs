//! Provides a RESTful web server managing some Todos.
//!
//! API will be:
//!
//! - `GET /todos`: return a JSON list of Todos.
//! - `POST /todos`: create a new Todo.
//! - `PUT /todos/:id`: update a specific Todo.
//! - `DELETE /todos/:id`: delete a specific Todo.
//!
//! Run with
//!
//! ```not_rust
//! cd examples && cargo run -p example-todos
//! ```

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::{IntoResponse},
    routing::{post,get_service},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    time::Duration,
    io
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tower_http::services::{ServeDir,ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_todos=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();


    // Compose the routes
    let app = Router::new()
        // 以下3个route很容易地就搭建起一个网站
        .route("/",get_service(ServeFile::new("./web_src/index.html")).handle_error(|error:io::Error|async move{
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error:{}",error),
            )
        }))
        .route("/suggestion",get_service(ServeFile::new("./web_src/suggestion.html")).handle_error(|error:io::Error|async move{
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error:{}",error),
            )
        }))
        .route("/:file/*tmp",get_service(ServeDir::new("./web_src")).handle_error(|error:io::Error|async move{
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error:{}",error),
            )
        }))
        .route("/push_text", post(get_text))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


#[derive(Debug, Deserialize)]
struct UserText {
    usertext: Option<String>,
    useremail: Option<String>
}
async fn get_text(
    Json(input): Json<UserText>
) -> impl IntoResponse {
    match input.usertext {
        None => println!("recieve empty text"),
        Some(push_str) => println!("the text is : {}",push_str)
    }
    match input.useremail {
        None => println!("recieve empty text"),
        Some(push_str) => println!("the email is : {}",push_str)
    }
    let return_text = Return{
        status:0,
        msg:String::from("save success"),
        data:Data { id: 1 }
    };
    (StatusCode::CREATED, Json(return_text))
}



#[derive(Debug, Serialize, Clone)]
struct Return {
    status: i64,
    msg: String,
    data: Data,
}

#[derive(Debug, Serialize, Clone)]
struct Data{
    id:i64,
}