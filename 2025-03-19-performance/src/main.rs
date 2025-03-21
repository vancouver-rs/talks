use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

#[inline(never)]
async fn handler() -> String {
    let response = "Hello, world!".to_string();

    // If you uncomment the following then you'll see that there are fewer samples in the
    // flamegraph. Why? Because by default it only includes samples from when your program is
    // active and not waiting.
    //tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Do something slow so it shows up in the flamegraph.
    let fib = bad_fib(20);
    let response = format!("{} {}", response, fib);

    // You can also uncomment the following to add an extra clone. This can be seen in the dtrace
    // example!
    //response.clone()
    response
}

#[inline(never)]
fn bad_fib(n: u32) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => bad_fib(n-1) + bad_fib(n-2),
    }
}
