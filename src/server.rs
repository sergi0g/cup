use std::sync::Arc;

use chrono::Local;
use json::JsonValue;
use liquid::{object, Object, ValueView};
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
    config::{Config, Theme},
    docker::get_images_from_docker_daemon,
    image::Image,
    info,
    utils::{sort_image_vec, timestamp, to_full_json, to_simple_json},
};

const HTML: &str = include_str!("static/index.html");
const JS: &str = include_str!("static/assets/index.js");
const CSS: &str = include_str!("static/assets/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

pub async fn serve(port: &u16, config: &Config) -> std::io::Result<()> {
    info!("Starting server, please wait...");
    let data = ServerData::new(config).await;
    info!("Ready to start!");
    App::new()
        .with_state(Arc::new(Mutex::new(data)))
        .at("/", get(handler_service(_static)))
        .at("/api/v1/simple", get(handler_service(api_simple)))
        .at("/api/v1/full", get(handler_service(api_full)))
        .at("/api/v1/refresh", get(handler_service(refresh)))
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

async fn api_simple(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::builder()
        .header("Content-Type", "application/json")
        .body(ResponseBody::from(json::stringify(
            data.lock().await.simple_json.clone(),
        )))
        .unwrap()
}

async fn api_full(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::builder()
        .header("Content-Type", "application/json")
        .body(ResponseBody::from(json::stringify(
            data.lock().await.full_json.clone(),
        )))
        .unwrap()
}

async fn refresh(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    data.lock().await.refresh().await;
    WebResponse::new(ResponseBody::from("OK"))
}

struct ServerData {
    template: String,
    raw_updates: Vec<Image>,
    simple_json: JsonValue,
    full_json: JsonValue,
    config: Config,
    theme: &'static str,
}

impl ServerData {
    async fn new(config: &Config) -> Self {
        let mut s = Self {
            config: config.clone(),
            template: String::new(),
            simple_json: JsonValue::Null,
            full_json: JsonValue::Null,
            raw_updates: Vec::new(),
            theme: "neutral",
        };
        s.refresh().await;
        s
    }
    async fn refresh(&mut self) {
        let start = timestamp();
        if !self.raw_updates.is_empty() {
            info!("Refreshing data");
        }
        let images = get_images_from_docker_daemon(&self.config, &None).await;
        let updates = sort_image_vec(&get_updates(&images, &self.config).await);
        let end = timestamp();
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
            .map(|image| object!({"name": image.reference, "has_update": image.has_update().to_option_bool().to_value()}),)
            .collect::<Vec<Object>>();
        self.simple_json = to_simple_json(&self.raw_updates);
        self.full_json = to_full_json(&self.raw_updates);
        let last_updated = Local::now();
        self.simple_json["last_updated"] = last_updated
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            .to_string()
            .into();
        self.full_json["last_updated"] = last_updated
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            .to_string()
            .into();
        self.theme = match &self.config.theme {
            Theme::Default => "neutral",
            Theme::Blue => "gray",
        };
        let globals = object!({
            "metrics": [{"name": "Monitored images", "value": self.simple_json["metrics"]["monitored_images"].as_usize()}, {"name": "Up to date", "value": self.simple_json["metrics"]["up_to_date"].as_usize()}, {"name": "Updates available", "value": self.simple_json["metrics"]["update_available"].as_usize()}, {"name": "Unknown", "value": self.simple_json["metrics"]["unknown"].as_usize()}],
            "images": images,
            "last_updated": last_updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "theme": &self.theme
        });
        self.template = template.render(&globals).unwrap();
    }
}
