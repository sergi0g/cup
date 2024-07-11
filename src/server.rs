use std::sync::Mutex;

use liquid::{object, Object};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use xitca_web::{
    body::ResponseBody,
    handler::{handler_service, state::StateOwn},
    http::WebResponse,
    route::get,
    App,
    middleware::Logger
};

const RAW_TEMPLATE: &str = include_str!("static/template.liquid");
const STYLE: &str = include_str!("static/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

pub async fn serve(port: &u16, updates: &[(String, Option<bool>)]) -> std::io::Result<()> {
    println!("Serving on http://0.0.0.0:{}", port);
    App::new()
        .with_state(updates.to_owned())
        .at("/", get(handler_service(home)))
        .at("/json", get(handler_service(json)))
        .at("/favicon.ico", handler_service(favicon_ico)) // These aren't pretty but this is xitca-web...
        .at("/favicon.svg", handler_service(favicon_svg))
        .at("/apple-touch-icon.png", handler_service(apple_touch_icon))
        .enclosed(Logger::new())
        .serve()
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .wait()
}

async fn home(
    updates: StateOwn<Vec<(String, Option<bool>)>>,
) -> WebResponse {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(RAW_TEMPLATE)
        .unwrap();
    let images = updates
        .0
        .par_iter()
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
    let globals = object!({
        "metrics": [{"name": "Monitored images", "value": images.len()}, {"name": "Up to date", "value": uptodate}, {"name": "Updates available", "value": updatable}, {"name": "Unknown", "value": unknown}],
        "images": images,
        "style": STYLE
    });
    let result = template.render(&globals).unwrap();
    WebResponse::new(ResponseBody::from(result))
}

async fn json(
    updates: StateOwn<Vec<(String, Option<bool>)>>
) -> WebResponse {
    let result_mutex: Mutex<json::object::Object> = Mutex::new(json::object::Object::new());
    updates.par_iter().for_each(|image| match image.1 {
        Some(b) => result_mutex.lock().unwrap().insert(&image.0, json::from(b)),
        None => result_mutex.lock().unwrap().insert(&image.0, json::Null),
    });
    let result = json::stringify(result_mutex.lock().unwrap().clone());
    WebResponse::new(ResponseBody::from(result))
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