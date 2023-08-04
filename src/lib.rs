use console_error_panic_hook;
use url::Url;
use worker::*;

mod slack;
use slack::LinkSharedEvent;

// https://github.com/cloudflare/workers-rs

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .get_async("/", |_req, _ctx| async move {
            console_log!("hallo");
            Response::ok("Get received")
        })
        .post_async("/", |mut req, _ctx| async move {
            let body: LinkSharedEvent = req.json().await?;
            console_log!("Received JSON: {:?}", body);
            let mut response_msg = String::from("Link object received successfully. Got IDs: ");
            for link in body.event.links {
                // Parse the shared links and get the creativeset and creative id
                if let Some(ids) = get_ids_from_url(&link.url) {
                    response_msg.push_str(&format!(", {:?}", ids));
                }
            }

            Response::ok(format!("JSON data logged successfully {:?}", response_msg))
        })
        .run(req, env)
        .await
}

fn get_ids_from_url(url: &str) -> Option<(u64, u64)> {
    let prefix = "/creatives";
    let parsed_url = Url::parse(url).ok()?;

    if parsed_url.path().starts_with(prefix) {
        let segments = parsed_url.path_segments()?.collect::<Vec<_>>();

        // segments  ["creatives", "12", "6666", "preview", ...]
        if segments.len() >= 3 && segments[0] == "creatives" {
            if let (Ok(id1), Ok(id2)) = (segments[1].parse::<u64>(), segments[2].parse::<u64>()) {
                return Some((id1, id2));
            }
        }
    }

    None
}
