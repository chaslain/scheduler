use lambda_http::{run, service_fn, Body, Error, Request, Response};


/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    
    let update: &String = match event.body() {
        Body::Empty => {
            return Ok(Response::builder().status(400).body("invalid".into()).map_err(Box::new)?)
        },
        Body::Text(req) => {
            req
        },
        Body::Binary(_) => {
            return Ok(Response::builder().status(400).body("invalid".into()).map_err(Box::new)?)
        }
    };

    println!("{}", update);
    
    let bot = telegram::BotBoy::new();

    bot.process_single_update_from_string(update);
    
    let resp = Response::builder()
        .status(200)
        .body("ok".into())
        .map_err(Box::new)?;
    Ok(resp)
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

    run(service_fn(function_handler)).await
}
