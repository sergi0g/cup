use std::sync::Arc;

use chrono::Local;
use json::JsonValue;
use liquid::{object, Object};
use tokio::sync::Mutex;
use xitca_web::{
    body::ResponseBody,
    handler::{handler_service, path::PathRef, state::StateRef},
    http::WebResponse,
    middleware::Logger,
    route::get,
    App,
};

use crate::{
    check::get_updates,
    docker::get_images_from_docker_daemon,
    error, info,
    utils::{sort_update_vec, to_json, CliConfig},
};

const HTML: &str = include_str!("static/index.html");
const JS: &str = include_str!("static/assets/index.js");
const CSS: &str = include_str!("static/assets/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

pub async fn serve(port: &u16, options: &CliConfig) -> std::io::Result<()> {
    info!("Starting server, please wait...");
    let data = ServerData::new(options).await;
    info!("Ready to start!");
    App::new()
        .with_state(Arc::new(Mutex::new(data)))
        .at("/", get(handler_service(_static)))
        .at("/json", get(handler_service(json)))
        .at("/refresh", get(handler_service(refresh)))
        .at("/*", get(handler_service(_static)))
        .enclosed(Logger::new())
        .serve()
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .wait()
}

async fn _static(data: StateRef<'_, Arc<Mutex<ServerData>>>, path: PathRef<'_>) -> WebResponse {
    match path.0 {
        "/" => WebResponse::builder()
            .header("Content-Type", "text/html")
            .body(ResponseBody::from(data.lock().await.template.clone()))
            .unwrap(),
        "/assets/index.js" => WebResponse::builder()
            .header("Content-Type", "text/javascript")
            .body(ResponseBody::from(JS.replace(
                "=\"neutral\"",
                &format!("=\"{}\"", data.lock().await.theme),
            )))
            .unwrap(),
        "/assets/index.css" => WebResponse::builder()
            .header("Content-Type", "text/css")
            .body(ResponseBody::from(CSS))
            .unwrap(),
        "/favicon.ico" => WebResponse::builder()
            .header("Content-Type", "image/vnd.microsoft.icon")
            .body(ResponseBody::from(FAVICON_ICO))
            .unwrap(),
        "/favicon.svg" => WebResponse::builder()
            .header("Content-Type", "image/svg+xml")
            .body(ResponseBody::from(FAVICON_SVG))
            .unwrap(),
        "/apple-touch-icon.png" => WebResponse::builder()
            .header("Content-Type", "image/png")
            .body(ResponseBody::from(APPLE_TOUCH_ICON))
            .unwrap(),
        _ => WebResponse::builder()
            .status(404)
            .body(ResponseBody::from("Not found"))
            .unwrap(),
    }
}

async fn json(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::new(ResponseBody::from(json::stringify(
        data.lock().await.json.clone(),
    )))
}

async fn refresh(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    data.lock().await.refresh().await;
    WebResponse::new(ResponseBody::from("OK"))
}

struct ServerData {
    template: String,
    raw_updates: Vec<(String, Option<bool>)>,
    json: JsonValue,
    options: CliConfig,
    theme: &'static str,
}

impl ServerData {
    async fn new(options: &CliConfig) -> Self {
        let mut s = Self {
            options: options.clone(),
            template: String::new(),
            json: json::object! {
                metrics: json::object! {},
                images: json::object! {},
            },
            raw_updates: Vec::new(),
            theme: "neutral",
        };
        s.refresh().await;
        s
    }
    async fn refresh(&mut self) {
        let start = Local::now().timestamp_millis();
        if !self.raw_updates.is_empty() {
            info!("Refreshing data");
        }
        let images = get_images_from_docker_daemon(&self.options, &None).await;
        let updates = sort_update_vec(&get_updates(&images, &self.options).await);
        let end = Local::now().timestamp_millis();
        info!("✨ Checked {} images in {}ms", updates.len(), end - start);
        self.raw_updates = updates;
        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(HTML)
            .unwrap();
        let images = self
            .raw_updates
            .iter()
            .map(|(name, has_update)| match has_update {
                Some(v) => object!({"name": name, "has_update": v.to_string()}), // Liquid kinda thinks false == nil, so we'll be comparing strings from now on
                None => object!({"name": name, "has_update": "null"}),
            })
            .collect::<Vec<Object>>();
        self.json = to_json(&self.raw_updates);
        let last_updated = Local::now();
        self.json["last_updated"] = last_updated
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            .to_string()
            .into();
        self.theme = match &self.options.config["theme"].as_str() {
            Some(t) => match *t {
                "default" => "neutral",
                "blue" => "gray",
                _ => error!(
                    "Invalid theme {} specified! Please choose between 'default' and 'blue'",
                    t
                ),
            },
            None => "neutral",
        };
        let globals = object!({
            "metrics": [{"name": "Monitored images", "value": self.json["metrics"]["monitored_images"].as_usize()}, {"name": "Up to date", "value": self.json["metrics"]["up_to_date"].as_usize()}, {"name": "Updates available", "value": self.json["metrics"]["update_available"].as_usize()}, {"name": "Unknown", "value": self.json["metrics"]["unknown"].as_usize()}],
            "images": images,
            "last_updated": last_updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "theme": &self.theme
        });
        self.template = template.render(&globals).unwrap();
    }
}
