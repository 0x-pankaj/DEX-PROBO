use poem::{
    get, handler, http::{ Method, StatusCode}, listener::TcpListener, middleware::Cors, post, web::{ Data, Json}, EndpointExt, IntoResponse, Response, Route, Server
};
use redis_lib::manager::RedisStore;
use serde::{Deserialize, Serialize};
use types::{
    api::MessageFromApi,
    order::{OptionType, OrderType},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::from_filename(".env").map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load .env file: {}", e),
        )
    })?;

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis = RedisStore::new(&redis_url).await.map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to initialize Redis: {}", e),
        )
    })?;

    let cors = Cors::new()
        .allow_method(Method::GET)
        .allow_method(Method::POST)
        .allow_method(Method::PATCH)
        .allow_method(Method::DELETE);

    let app = Route::new()
    .at("/place_order", post(place_order))
    .at("ping", get(pong))        
    .with(cors).data(redis);

    Server::new(TcpListener::bind("0.0.0.0:8000"))
        
        .run(app)
        .await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String
}

//function to create error response
fn error_response(error: String, status: StatusCode ) -> Response {
    let error_resp = ErrorResponse {error};
    Response::builder().status(status)
        .content_type("application/json")
        .body(serde_json::to_string(&error_resp).unwrap_or_default())
}
// function to create success response
fn success_response<T: serde::Serialize>(data: T) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&data).unwrap_or_default())
}

#[handler]
async fn pong() -> impl IntoResponse {
    success_response("pong".to_string())
}

#[derive(Deserialize, Serialize, Clone)]
struct PlaceOrderRequest {
    user_id: String,
    market_id: String,
    option: String,
    order_type: String,
    price: f64,
    quantity: u32,
}

#[handler]
async fn place_order(redis: Data<&RedisStore>, req: Json<PlaceOrderRequest>) -> impl IntoResponse {
    // Validate the request
    if req.user_id.is_empty() || req.market_id.is_empty() || req.option.is_empty() ||
       req.order_type.is_empty() || req.price <= 0.0 || req.quantity == 0 {
        return error_response("Invalid request parameters".to_string(), StatusCode::BAD_REQUEST);
    }
    let option = match req.option.as_str() {
        "Yes" => OptionType::Yes,
        "No" => OptionType::No,
        _ => return error_response("Invalid option".to_string(), StatusCode::BAD_REQUEST),
    };

    let order_type = match req.order_type.as_str() {
        "Buy" => OrderType::Buy,
        "Sell" => OrderType::Sell,
        _ => return error_response("Invalid order type".to_string(), StatusCode::BAD_REQUEST),
    };

    let order = MessageFromApi::CreateOrder {
        user_id: req.user_id.clone(),
        marker_id: req.market_id.clone(),
        option: option,
        order_type: order_type,
        price: req.price,
        quantity: req.quantity,
    };

    //Handle placing the order in Redis
    match redis.add_message_to_stream(&order).await {
        Ok(_) => success_response("Order placed successfully".to_string()),
        Err(e) => {
            return error_response(format!("Failed to place order: {}", e), StatusCode::INTERNAL_SERVER_ERROR);
        }
        
    }
}
