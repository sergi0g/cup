use std::{collections::HashMap, sync::Arc};

use chrono::Local;
use liquid::{object, Object};
use tokio::sync::Mutex;
use xitca_web::{
    body::ResponseBody,
    handler::{handler_service, state::StateRef},
    http::WebResponse,
    middleware::Logger,
    route::get,
    App,
};

use crate::{
    check::get_all_updates,
    error,
    utils::{sort_update_vec, to_json, Config, JsonData},
};

const RAW_TEMPLATE: &str = include_str!("static/template.liquid");
const STYLE: &str = include_str!("static/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

pub async fn serve(port: &u16, socket: Option<String>, config: Config) -> std::io::Result<()> {
    let mut data = ServerData::new(socket, config).await;
    data.refresh().await;
    App::new()
        .with_state(Arc::new(Mutex::new(data)))
        .at("/", get(handler_service(home)))
        .at("/json", get(handler_service(json)))
        .at("/refresh", get(handler_service(refresh)))
        .at("/favicon.ico", handler_service(favicon_ico)) // These aren't pretty but this is xitca-web...
        .at("/favicon.svg", handler_service(favicon_svg))
        .at("/apple-touch-icon.png", handler_service(apple_touch_icon))
        .enclosed(Logger::new())
        .serve()
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .wait()
}

async fn home(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::new(ResponseBody::from(data.lock().await.template.clone()))
}

async fn json(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::new(ResponseBody::from(
        serde_json::to_string(&data.lock().await.json).unwrap(),
    ))
}

async fn refresh(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    data.lock().await.refresh().await;
    WebResponse::new(ResponseBody::from("OK"))
}

async fn favicon_ico() -> WebResponse {
    WebResponse::new(ResponseBody::from(FAVICON_ICO))
}

async fn favicon_svg() -> WebResponse {
    WebResponse::new(ResponseBody::from(FAVICON_SVG))
}

async fn apple_touch_icon() -> WebResponse {
    WebResponse::new(ResponseBody::from(APPLE_TOUCH_ICON))
}

struct ServerData {
    template: String,
    raw_updates: Vec<(String, Option<bool>)>,
    json: JsonData,
    socket: Option<String>,
    config: Config,
}

impl ServerData {
    async fn new(socket: Option<String>, config: Config) -> Self {
        let mut s = Self {
            socket,
            template: String::new(),
            json: JsonData {
                metrics: HashMap::new(),
                images: HashMap::new(),
            },
            raw_updates: Vec::new(),
            config,
        };
        s.refresh().await;
        s
    }
    async fn refresh(&mut self) {
        let updates = sort_update_vec(&get_all_updates(self.socket.clone()).await);
        self.raw_updates = updates;
        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(RAW_TEMPLATE)
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
        let last_updated = Local::now().format("%Y-%m-%d %H:%M:%S");
        let theme = match &self.config.theme {
            Some(t) => match t.as_str() {
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
            "metrics": [{"name": "Monitored images", "value": self.json.metrics.get("monitored_images")}, {"name": "Up to date", "value": self.json.metrics.get("up_to_date")}, {"name": "Updates available", "value": self.json.metrics.get("update_available")}, {"name": "Unknown", "value": self.json.metrics.get("unknown")}],
            "images": images,
            "style": STYLE,
            "last_updated": last_updated.to_string(),
            "theme": theme
        });
        self.template = template.render(&globals).unwrap();
    }
}
