use std::env;
use std::net::Ipv4Addr;
use warp::{http::Response, Filter};

// Import the LinkObject struct from the link_object module
mod slack;
use slack::LinkSharedEvent;

#[tokio::main]
async fn main() {
    // Define a route to handle POST requests to '/'
    let example1 = warp::post()
        .and(warp::path::end()) // Match only the root path '/'
        .and(warp::body::json()) // Extract the JSON body
        .map(|link: LinkSharedEvent| {
            // Process the link object received in the request body
            // For now, let's just print it to the console
            println!("Received link: {:?}", link);

            // Return a response indicating success
            Response::builder().body(String::from("Link object received successfully."))
        });

    // Rest of your code remains unchanged

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    warp::serve(example1).run((Ipv4Addr::LOCALHOST, port)).await
}
