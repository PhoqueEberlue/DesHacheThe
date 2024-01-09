// WARP
use warp::Filter;
use tokio::io::{self, AsyncBufReadExt};

pub async fn run_rest_api() {
    let routes = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let server = warp::serve(routes);
    
    // Spawn the server into a runtime
    tokio::task::spawn(server.run(([127, 0, 0, 1], 3030)));
}
// REST API
    
