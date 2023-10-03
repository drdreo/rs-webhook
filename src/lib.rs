use console_error_panic_hook;
use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;
use url::Url;
use worker::*;

mod slack;
use slack::LinkSharedEvent;
use slack::SlackEvent;
use slack::UrlVerificationEvent;

// https://github.com/cloudflare/workers-rs
#[event(fetch)]
pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let method = &req.method();
    match method {
        Method::Get => Response::ok("GET /"),
        Method::Post => {
            let json = req.text().await?;
            let event = parse_slack_event(req.headers(), json);
            console_log!("Received JSON: {:?}", event);

            return match event {
                Ok(event) => match event {
                    SlackEvent::UrlVerification(auth_evt) => Response::ok(auth_evt.challenge),
                    SlackEvent::LinkShared(share_evt) => {
                        handle_link_shared_event(share_evt.clone(), env)
                            .await
                            .unwrap();
                        Response::ok(share_evt.token) // Return the early response
                    }
                },
                Err(_) => Response::error("Bad Request", 400),
            };
        }
        _ => Response::ok("Method Not Supported"),
    }
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

fn parse_slack_event(headers: &Headers, json: String) -> Result<SlackEvent> {
    let event: Value = serde_json::from_str(&json)?;

    if is_slack_auth_event(&headers, &event) {
        let auth_evt: UrlVerificationEvent = serde_json::from_value(event)?;
        Ok(SlackEvent::UrlVerification(auth_evt))
    } else if is_slack_link_shared_event(&headers, &event) {
        let link_shared_event: LinkSharedEvent = serde_json::from_value(event)?;
        Ok(SlackEvent::LinkShared(link_shared_event))
    } else {
        Err("Unknown event type".into())
    }
}

fn is_slack_auth_event(headers: &Headers, json: &Value) -> bool {
    headers.has("x-slack-request-timestamp").unwrap() && json.get("challenge").is_some()
}

fn is_slack_link_shared_event(headers: &Headers, json: &Value) -> bool {
    headers.has("x-slack-request-timestamp").unwrap() && json.get("event").is_some()
}

async fn handle_link_shared_event(ls_evt: LinkSharedEvent, env: Env) -> Result<()> {
    let mut response_msg = String::from("Link object received successfully. Got IDs: ");

    let link = &ls_evt.event.links[0];
    // Parse the shared links and get the creativeset and creative id
    if let Some((creativeset, creative)) = get_ids_from_url(&link.url) {
        response_msg.push_str(&format!(", {:?}", (creativeset, creative)));

        // check if the link is referencing sandbox or production and use proper ACG instance
        // TODO: add snapshot it here
        let url = if link.url.contains("sandbox-studio.bannerflow.com") {
            format!("https://sandbox-studio.bannerflow.com/creatives/creative-metadata?creativeset={:?}&creative={:?}", creativeset, creative)
        } else {
            format!("https://studio.bannerflow.com/creatives/creative-metadata?creativeset={:?}&creative={:?}", creativeset, creative)
        };

        let response = reqwest::get(&url).await;
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.text().await.unwrap();
                    let meta: Value = serde_json::from_str(&content)?;
                    console_log!("Creative meta: {}", meta);
                    send_slack_unfurl_request(meta, &link.url, &ls_evt, creativeset, creative, env)
                        .await?;
                } else {
                    console_error!("Request failed with status code: {}", response.status());
                }
            }
            Err(reqwest_err) => {
                return Err(worker::Error::from(format!(
                    "Reqwest error: {}",
                    reqwest_err
                )));
            }
        }
    }

    console_log!("Unfurl request received successfully. {}", response_msg);
    Ok(())
}

async fn send_slack_unfurl_request(
    meta: Value,
    shared_link: &str,
    event: &LinkSharedEvent,
    creativeset: u64,
    creative: u64,
    env: Env,
) -> Result<()> {
    let mut shared_links: HashMap<String, Value> = HashMap::new();
    let is_sandbox = shared_link.contains("sandbox-studio.bannerflow.com");

    let mv_link = if is_sandbox {
        format!(
            "https://sandbox-studio.bannerflow.com/brand/{}/creativeset/{}",
            meta["brand"].as_str().unwrap(),
            creativeset
        )
    } else {
        format!(
            "https://studio.bannerflow.com/brand/{}/creativeset/{}",
            meta["brand"].as_str().unwrap(),
            creativeset
        )
    };

    shared_links.insert(shared_link.to_string(), json!({
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": "Creative Preview - Bannerflow",
                        "emoji": true
                    }
                },
                {
                    "type": "section",
                    "fields": [
                        {
                            "type": "mrkdwn",
                            "text": format!("*Creativeset:* {}\n{}", meta["creativeset"], creativeset)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Creative:*\n{}", creative)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Size:*\n{} x {}", meta["size"]["width"], meta["size"]["height"])
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Version:*\n{}", meta["version"].as_str().unwrap())
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Brand:*\n{}", meta["brand"].as_str().unwrap())
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Elements:* {}\n*Duration:* {}", meta["elements"], meta["duration"])
                        }
                    ],
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("<{}|View preview>", shared_link),
                    },
                    "accessory": {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "Go To MV",
                            "emoji": true
                        },
                        "url": mv_link,
                        "action_id": "button-action",
                    },
                },
                {
                    "type": "image",
                    "title": {
                        "type": "plain_text",
                        "text": format!("{} - {}", creativeset, creative),
                        "emoji": true,
                    },
                    "image_url": get_image_url(meta["preloadImage"].as_str().unwrap()),
                    "alt_text": "preload image",
                },
            ],
        }));

    let res_body = json!({
        "channel": event.event.channel,
        "ts": event.event.message_ts,
        "unfurls": shared_links,
    });

    console_log!("Unfurl data: {}", res_body);

    let bot_token = env
        .secret("BOT_TOKEN")
        .expect("BOT_TOKEN environment variable not found");

    let res = reqwest::Client::new()
        .post("https://slack.com/api/chat.unfurl")
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Authorization", format!("Bearer {}", bot_token.to_string()))
        .json(&res_body)
        .send()
        .await
        .unwrap();

    let res_json: Value = res.json().await.unwrap();

    if res_json["ok"].as_bool().unwrap_or(false) {
        console_log!("Unfurl successful");
    } else {
        console_warn!("Unfurl unsuccessful: {}", res_json);
    }

    Ok(())
}

fn get_image_url(url: &str) -> String {
    format!(
        "https://c.bannerflow.net/io/api/image/optimize?u={}&w=200&h=200&q=85&f=webp&rt=contain",
        url
    )
}
