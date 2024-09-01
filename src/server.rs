use std::sync::Arc;

use chrono::Local;
use json::JsonValue;
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
    check::get_all_updates,
    error,
    utils::{sort_update_vec, to_json},
};

const HTML: &str = include_str!("static/index.html");
const JS: &str = include_str!("static/assets/index.js");
const CSS: &str = include_str!("static/assets/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

pub async fn serve(port: &u16, socket: Option<String>, config: JsonValue) -> std::io::Result<()> {
    println!("Starting server, please wait...");
    let data = ServerData::new(socket, config).await;
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
        "/" => WebResponse::builder().header("Content-Type", "text/html").body(ResponseBody::from(HTML)).unwrap(),
        "/assets/index.js" => WebResponse::builder().header("Content-Type", "text/javascript").body(ResponseBody::from(JS.replace("=\"neutral\"", &format!("=\"{}\"", data.lock().await.theme)))).unwrap(),
        "/assets/index.css" => WebResponse::builder().header("Content-Type", "text/css").body(ResponseBody::from(CSS)).unwrap(),
        "/favicon.ico" => WebResponse::builder().header("Content-Type", "image/vnd.microsoft.icon").body(ResponseBody::from(FAVICON_ICO)).unwrap(),
        "/favicon.svg" => WebResponse::builder().header("Content-Type", "image/svg+xml").body(ResponseBody::from(FAVICON_SVG)).unwrap(),
        "/apple-touch-icon.png" => WebResponse::builder().header("Content-Type", "image/png").body(ResponseBody::from(APPLE_TOUCH_ICON)).unwrap(),
        _ => WebResponse::builder().status(404).body(ResponseBody::from("Not found")).unwrap()
    }
}

async fn json(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::new(ResponseBody::from(
        json::stringify(data.lock().await.json.clone())
    ))
}

async fn refresh(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    data.lock().await.refresh().await;
    WebResponse::new(ResponseBody::from("OK"))
}

struct ServerData {
    raw_updates: Vec<(String, Option<bool>)>,
    json: JsonValue,
    socket: Option<String>,
    config: JsonValue,
    theme: &'static str
}

impl ServerData {
    async fn new(socket: Option<String>, config: JsonValue) -> Self {
        let mut s = Self {
            socket,
            json: json::object! {
                metrics: json::object! {},
                images: json::object! {},
            },
            raw_updates: Vec::new(),
            config,
            theme: "neutral"
        };
        s.refresh().await;
        s
    }
    async fn refresh(&mut self) {
        let updates = sort_update_vec(&get_all_updates(self.socket.clone(), &self.config["authentication"]).await);
        self.raw_updates = updates;
        self.json = to_json(&self.raw_updates);
        let last_updated = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.json["last_updated"] = last_updated.to_string().into();
        self.theme = match &self.config["theme"].as_str() {
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
    }
}
