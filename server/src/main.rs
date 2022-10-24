use warp::Filter;
use telegram::{Update, BotBoy};

#[tokio::main]
async fn main() {
    let entry 
    = warp::path::end()
    .and(warp::body::json())
    .and(warp::post())
    .map(go);

    warp::serve(entry).run(([0,0,0,0], 80)).await;
}

fn go(update: Update) -> String{

    let mut vec = Vec::new();
    let bot = BotBoy::new();

    vec.push(update);
    bot.process_updates_vec(vec);
    "200 OK".to_owned()
}