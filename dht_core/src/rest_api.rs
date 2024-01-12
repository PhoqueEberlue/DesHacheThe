// WARP
use warp::Filter;

pub(crate) async fn run() {
    let routes = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let server = warp::serve(routes);
    
    println!("Rest API running");
    // Spawn the server into a runtime
    let _ = server.run(([127, 0, 0, 1], 3030)).await;
}
