use std::sync::{Arc, Mutex};

use chrono::Local;
use liquid::{object, Object};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use xitca_web::{
    body::ResponseBody,
    handler::{handler_service, state::StateRef},
    http::WebResponse,
    middleware::Logger,
    route::get,
    App,
};

use crate::{check::get_all_updates, utils::{sort_update_vec, Config}};

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
    WebResponse::new(ResponseBody::from(data.lock().unwrap().template.clone()))
}

async fn json(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::new(ResponseBody::from(data.lock().unwrap().json.clone()))
}

async fn refresh(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    data.lock().unwrap().refresh().await;
    return WebResponse::new(ResponseBody::from("OK"));
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
    json: String,
    socket: Option<String>,
    config: Config,
}

impl ServerData {
    async fn new(socket: Option<String>, config: Config) -> Self {
        return Self {
            socket,
            template: String::new(),
            json: String::new(),
            raw_updates: Vec::new(),
            config,
        };
    }
    async fn refresh(self: &mut Self) {
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
            .map(|(name, image)| match image {
                Some(value) => {
                    if *value {
                        object!({"name": name, "status": "update-available"})
                    } else {
                        object!({"name": name, "status": "up-to-date"})
                    }
                }
                None => object!({"name": name, "status": "unknown"}),
            })
            .collect::<Vec<Object>>();
        let uptodate = images
            .par_iter()
            .filter(|&o| o["status"] == "up-to-date")
            .collect::<Vec<&Object>>()
            .len();
        let updatable = images
            .par_iter()
            .filter(|&o| o["status"] == "update-available")
            .collect::<Vec<&Object>>()
            .len();
        let unknown = images
            .par_iter()
            .filter(|&o| o["status"] == "unknown")
            .collect::<Vec<&Object>>()
            .len();
        let last_updated = Local::now().format("%Y-%m-%d %H:%M:%S");
        let globals = object!({
            "metrics": [{"name": "Monitored images", "value": images.len()}, {"name": "Up to date", "value": uptodate}, {"name": "Updates available", "value": updatable}, {"name": "Unknown", "value": unknown}],
            "images": images,
            "style": STYLE,
            "last_updated": last_updated.to_string(),
            "theme": self.config.theme
        });
        self.template = template.render(&globals).unwrap();
        let json_data: Mutex<json::object::Object> = Mutex::new(json::object::Object::new());
        self.raw_updates.par_iter().for_each(|image| match image.1 {
            Some(b) => json_data.lock().unwrap().insert(&image.0, json::from(b)),
            None => json_data.lock().unwrap().insert(&image.0, json::Null),
        });
        self.json = json::stringify(json_data.lock().unwrap().clone());
    }
}
