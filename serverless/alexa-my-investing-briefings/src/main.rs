// This example requires the following input to succeed:
// { "command": "do something" }

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb as dynamodb;
use dynamodb::model::AttributeValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// This is also a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    user_name: String,
}

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // extract some useful info from the request
    let user_name = event.payload.user_name;

    let is_success = insert_user(&user_name).await;

    let txt_response = if is_success {
        format!("Created user: {}", user_name)
    } else {
        format!("Failed to create user: {}", user_name)
    };

    // prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: txt_response,
    };

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

async fn insert_user(user_name: &String) -> bool {
    let region_provider = RegionProviderChain::default_provider().or_else("sa-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = dynamodb::Client::new(&config);

    let uuid = Uuid::new_v4().to_string();

    let request = client
        .put_item()
        .table_name("tb_users")
        .item("user_id", AttributeValue::S(String::from(uuid)))
        .item("user_name", AttributeValue::S(String::from(user_name)));

    let result = request.send().await;

    result.is_ok()
}
