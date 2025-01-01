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
    info,
    structs::image::Image,
    utils::{
        json::{to_full_json, to_simple_json},
        misc::timestamp,
        sort_update_vec::sort_image_vec,
    },
};

const HTML: &str = include_str!("static/index.html");
const JS: &str = include_str!("static/assets/index.js");
const CSS: &str = include_str!("static/assets/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

const SORT_ORDER: [&'static str; 8] = [
    "monitored_images",
    "updates_available",
    "major_updates",
    "minor_updates",
    "patch_updates",
    "other_updates",
    "up_to_date",
    "unknown",
]; // For Liquid rendering

pub async fn serve(port: &u16, config: &Config) -> std::io::Result<()> {
    info!("Starting server, please wait...");
    let data = ServerData::new(config).await;
    info!("Ready to start!");
    let mut app_builder = App::new()
        .with_state(Arc::new(Mutex::new(data)))
        .at("/api/v2/json", get(handler_service(api_simple)))
        .at("/api/v3/json", get(handler_service(api_full)))
        .at("/api/v2/refresh", get(handler_service(refresh)))
        .at("/api/v3/refresh", get(handler_service(refresh)));
    if !config.agent {
        app_builder = app_builder
            .at("/", get(handler_service(_static)))
            .at("/*", get(handler_service(_static)));
    }
    app_builder
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
        let updates = sort_image_vec(&get_updates(&None, &self.config).await);
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
            .map(|image| object!({"name": image.reference, "status": image.has_update().to_string()}),)
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
        let mut metrics = self.simple_json["metrics"].entries().map(|(key, value)| liquid::object!({ "name": key, "value": value.as_u16().unwrap()})).collect::<Vec<_>>();
        metrics.sort_unstable_by(|a, b| {dbg!(a, b); SORT_ORDER.iter().position(|i| i == &a["name"].to_kstr().as_str()).unwrap().cmp(&SORT_ORDER.iter().position(|i| i == &b["name"].to_kstr().as_str()).unwrap())});
        let globals = object!({
            "metrics": metrics,
            "images": images,
            "last_updated": last_updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "theme": &self.theme
        });
        self.template = template.render(&globals).unwrap();
    }
}
