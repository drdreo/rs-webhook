use std::env;
use std::net::Ipv4Addr;
use url::Url;
use warp::{http::Response, Filter};

mod slack;
use slack::LinkSharedEvent;

#[tokio::main]
async fn main() {
    let auth_route = warp::get()
        .and(warp::path::end())
        .map(|| Response::builder().body(String::from("This is the GET route response.")));

    let hook_route = warp::post()
        .and(warp::path::end())
        .and(warp::body::json())
        .map(|evt: LinkSharedEvent| {
            println!("Received event");

            let mut response_msg = String::from("Link object received successfully. Got IDs: ");

            for link in evt.event.links {
                // Parse the shared links and get the creativeset and creative id
                if let Some(ids) = get_ids_from_url(&link.url) {
                    response_msg.push_str(&format!(", {:?}", ids));
                }
            }

            Response::builder().body(response_msg)
        });

    let combined_routes = auth_route.or(hook_route);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    println!("Starting server on {:?}:{:?}", Ipv4Addr::LOCALHOST, port);
    warp::serve(combined_routes)
        .run((Ipv4Addr::LOCALHOST, port))
        .await
}

fn get_ids_from_url(url: &str) -> Option<(u64, u64)> {
    let prefix = "/creatives";
    let parsed_url = Url::parse(url).ok()?;

    if parsed_url.path().starts_with(prefix) {
        let segments = parsed_url.path_segments()?.collect::<Vec<_>>();

        // segments  ["creatives", "12", "6666", "preview"]
        if segments.len() >= 3 && segments[0] == "creatives" {
            if let (Ok(id1), Ok(id2)) = (segments[1].parse::<u64>(), segments[2].parse::<u64>()) {
                return Some((id1, id2));
            }
        }
    }

    None
}
