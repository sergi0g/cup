use std::{env, sync::Arc};

use chrono::Local;
use chrono_tz::Tz;
use liquid::{object, Object, ValueView};
use rustc_hash::FxHashMap;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use xitca_web::{
    body::ResponseBody,
    bytes::Bytes,
    error::Error,
    handler::{handler_service, path::PathRef, state::StateRef},
    http::{StatusCode, WebResponse},
    route::get,
    service::Service,
    App, WebContext,
};

use crate::{
    check::get_updates,
    config::Theme,
    error,
    structs::update::Update,
    utils::{
        json::{to_full_json, to_simple_json},
        sort_update_vec::sort_update_vec,
        time::{elapsed, now},
    },
    Context,
};

const HTML: &str = include_str!("static/index.html");
const JS: &str = include_str!("static/assets/index.js");
const CSS: &str = include_str!("static/assets/index.css");
const FAVICON_ICO: Bytes = Bytes::from_static(include_bytes!("static/favicon.ico"));
const FAVICON_SVG: Bytes = Bytes::from_static(include_bytes!("static/favicon.svg"));
const APPLE_TOUCH_ICON: Bytes = Bytes::from_static(include_bytes!("static/apple-touch-icon.png"));

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

pub async fn serve(port: &u16, ctx: &Context) -> std::io::Result<()> {
    ctx.logger.info("Starting server, please wait...");
    let data = ServerData::new(ctx).await;
    let scheduler = JobScheduler::new().await.unwrap();
    let data = Arc::new(Mutex::new(data));
    let data_copy = data.clone();
    let tz = env::var("TZ")
        .map(|tz| tz.parse().unwrap_or(Tz::UTC))
        .unwrap_or(Tz::UTC);
    if let Some(interval) = &ctx.config.refresh_interval {
        scheduler
            .add(
                match Job::new_async_tz(
                    interval,
                    tz,
                    move |_uuid, _lock| {
                        let data_copy = data_copy.clone();
                        Box::pin(async move {
                            data_copy.lock().await.refresh().await;
                        })
                    },
                ) {
                    Ok(job) => job,
                    Err(e) => match e {
                        tokio_cron_scheduler::JobSchedulerError::ParseSchedule => error!(
                            "Failed to parse cron schedule: {}. Please ensure it is valid!",
                            interval
                        ),
                        e => error!(
                            "An unexpected error occured while scheduling automatic refresh: {}",
                            e
                        ),
                    },
                },
            )
            .await
            .unwrap();
    }
    scheduler.start().await.unwrap();
    ctx.logger.info("Ready to start!");
    let mut app_builder = App::new()
        .with_state(data)
        .at("/api/v2/json", get(handler_service(api_simple)))
        .at("/api/v3/json", get(handler_service(api_full)))
        .at("/api/v2/refresh", get(handler_service(refresh)))
        .at("/api/v3/refresh", get(handler_service(refresh)));
    if !ctx.config.agent {
        app_builder = app_builder
            .at("/", get(handler_service(_static)))
            .at("/*", get(handler_service(_static)));
    }
    match app_builder
        .enclosed_fn(logger)
        .serve()
        .bind(format!("0.0.0.0:{}", port))
    {
        Ok(r) => r,
        Err(_) => error!("Failed to bind to port {}. Is it in use?", port),
    }
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
    raw_updates: Vec<Update>,
    simple_json: Value,
    full_json: Value,
    ctx: Context,
    theme: &'static str,
}

impl ServerData {
    async fn new(ctx: &Context) -> Self {
        let mut s = Self {
            ctx: ctx.clone(),
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
            self.ctx.logger.info("Refreshing data");
        }
        let updates = sort_update_vec(&get_updates(&None, true, &self.ctx).await);
        self.ctx.logger.info(format!(
            "✨ Checked {} images in {}ms",
            updates.len(),
            elapsed(start)
        ));
        self.raw_updates = updates;
        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(HTML)
            .unwrap();
        self.simple_json = to_simple_json(&self.raw_updates);
        self.full_json = to_full_json(&self.raw_updates);
        let last_updated = Local::now();
        self.simple_json["last_updated"] = last_updated
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            .to_string()
            .into();
        self.full_json["last_updated"] = self.simple_json["last_updated"].clone();
        self.theme = match &self.ctx.config.theme {
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
        let mut servers: FxHashMap<&str, Vec<Object>> = FxHashMap::default();
        self.raw_updates.iter().for_each(|update| {
            let key = update.server.as_deref().unwrap_or("");
            match servers.get_mut(&key) {
                Some(server) => server.push(
                    object!({"name": update.reference, "status": update.get_status().to_string()}),
                ),
                None => {
                    let _ = servers.insert(key, vec![object!({"name": update.reference, "status": update.get_status().to_string()})]);
                }
            }
        });
        let globals = object!({
            "metrics": metrics,
            "servers": servers,
            "server_ids": servers.into_keys().collect::<Vec<&str>>(),
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
        "\x1b[94;1m HTTP \x1b[0m\x1b[32m{}\x1b[0m {} {}{}\x1b[0m in {}ms",
        method, url, color, status, time
    )
}
