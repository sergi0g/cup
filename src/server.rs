use std::sync::Mutex;

use liquid::{object, Object};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use xitca_web::{
    body::ResponseBody,
    handler::{handler_service, path::PathOwn, state::StateOwn},
    http::{Method, WebResponse},
    route::get,
    App,
};

const RAW_TEMPLATE: &str = include_str!("template.liquid");
const STYLE: &str = include_str!("index.css");

pub async fn serve(port: &u16, updates: &[(String, Option<bool>)]) -> std::io::Result<()> {
    println!("Serving on http://0.0.0.0:{}", port);
    App::new()
        .with_state(updates.to_owned())
        .at("/", get(handler_service(home)))
        .at("/json", get(handler_service(json)))
        .serve()
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .wait()
}

async fn home(
    updates: StateOwn<Vec<(String, Option<bool>)>>,
    method: Method,
    path: PathOwn,
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
    println!("Received {} request on {}", method, path.0);
    WebResponse::new(ResponseBody::from(result))
}

async fn json(
    updates: StateOwn<Vec<(String, Option<bool>)>>,
    method: Method,
    path: PathOwn,
) -> WebResponse {
    let result_mutex: Mutex<json::object::Object> = Mutex::new(json::object::Object::new());
    updates.par_iter().for_each(|image| match image.1 {
        Some(b) => result_mutex.lock().unwrap().insert(&image.0, json::from(b)),
        None => result_mutex.lock().unwrap().insert(&image.0, json::Null),
    });
    let result = json::stringify(result_mutex.lock().unwrap().clone());
    println!("Received {} request on {}", method, path.0);
    WebResponse::new(ResponseBody::from(result))
}
