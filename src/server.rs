use std::sync::Arc;

use chrono::Local;
use liquid::{object, Object, ValueView};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use xitca_web::{
    body::ResponseBody,
    error::Error,
    handler::{handler_service, path::PathRef, state::StateRef},
    http::{StatusCode, WebResponse},
    route::get,
    service::Service,
    App, WebContext,
};

use crate::{
    check::get_updates,
    config::{Config, Theme},
    info,
    structs::image::Image,
    utils::{
        json::{to_full_json, to_simple_json},
        sort_update_vec::sort_image_vec,
        time::{elapsed, now},
    },
};

const HTML: &str = include_str!("static/index.html");
const JS: &str = include_str!("static/assets/index.js");
const CSS: &str = include_str!("static/assets/index.css");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("static/apple-touch-icon.png");

const SORT_ORDER: [&str; 8] = [
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
    let scheduler = JobScheduler::new().await.unwrap();
    let data = Arc::new(Mutex::new(data));
    let data_copy = data.clone();
    if let Some(interval) = &config.refresh_interval {
        scheduler
            .add(
                Job::new_async(interval, move |_uuid, _lock| {
                    let data_copy = data_copy.clone();
                    Box::pin(async move {
                        data_copy.lock().await.refresh().await;
                    })
                })
                .unwrap(),
            )
            .await
            .unwrap();
    }
    scheduler.start().await.unwrap();
    info!("Ready to start!");
    let mut app_builder = App::new()
        .with_state(data)
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
        .enclosed_fn(logger)
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
        .body(ResponseBody::from(
            data.lock().await.simple_json.clone().to_string(),
        ))
        .unwrap()
}

async fn api_full(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    WebResponse::builder()
        .header("Content-Type", "application/json")
        .body(ResponseBody::from(
            data.lock().await.full_json.clone().to_string(),
        ))
        .unwrap()
}

async fn refresh(data: StateRef<'_, Arc<Mutex<ServerData>>>) -> WebResponse {
    data.lock().await.refresh().await;
    WebResponse::new(ResponseBody::from("OK"))
}

struct ServerData {
    template: String,
    raw_updates: Vec<Image>,
    simple_json: Value,
    full_json: Value,
    config: Config,
    theme: &'static str,
}

impl ServerData {
    async fn new(config: &Config) -> Self {
        let mut s = Self {
            config: config.clone(),
            template: String::new(),
            simple_json: Value::Null,
            full_json: Value::Null,
            raw_updates: Vec::new(),
            theme: "neutral",
        };
        s.refresh().await;
        s
    }
    async fn refresh(&mut self) {
        let start = now();
        if !self.raw_updates.is_empty() {
            info!("Refreshing data");
        }
        let updates = sort_image_vec(&get_updates(&None, &self.config).await);
        info!(
            "✨ Checked {} images in {}ms",
            updates.len(),
            elapsed(start)
        );
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
        self.full_json["last_updated"] = self.simple_json["last_updated"].clone();
        self.theme = match &self.config.theme {
            Theme::Default => "neutral",
            Theme::Blue => "gray",
        };
        let mut metrics = self.simple_json["metrics"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| liquid::object!({ "name": key, "value": value }))
            .collect::<Vec<_>>();
        metrics.sort_unstable_by(|a, b| {
            SORT_ORDER
                .iter()
                .position(|i| i == &a["name"].to_kstr().as_str())
                .unwrap()
                .cmp(
                    &SORT_ORDER
                        .iter()
                        .position(|i| i == &b["name"].to_kstr().as_str())
                        .unwrap(),
                )
        });
        let globals = object!({
            "metrics": metrics,
            "images": images,
            "last_updated": last_updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "theme": &self.theme
        });
        self.template = template.render(&globals).unwrap();
    }
}

async fn logger<S, C, B>(next: &S, ctx: WebContext<'_, C, B>) -> Result<WebResponse, Error<C>>
where
    S: for<'r> Service<WebContext<'r, C, B>, Response = WebResponse, Error = Error<C>>,
{
    let start = now();
    let request = ctx.req();
    let method = request.method().to_string();
    let url = request.uri().to_string();

    if &method != "GET" {
        // We only allow GET requests

        log(&method, &url, 405, elapsed(start));
        Err(Error::from(StatusCode::METHOD_NOT_ALLOWED))
    } else {
        let res = next.call(ctx).await?;
        let status = res.status().as_u16();

        log(&method, &url, status, elapsed(start));
        Ok(res)
    }
}

fn log(method: &str, url: &str, status: u16, time: u32) {
    let color = {
        if status == 200 {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        }
    };
    println!(
        "\x1b[94;1mHTTP  \x1b[0m\x1b[32m{}\x1b[0m {} {}{}\x1b[0m in {}ms",
        method, url, color, status, time
    )
}
