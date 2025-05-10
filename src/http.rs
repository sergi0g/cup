use std::fmt::Display;

use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use crate::{error, Context};

pub enum RequestMethod {
    GET,
    HEAD,
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RequestMethod::GET => "GET",
            RequestMethod::HEAD => "HEAD",
        })
    }
}

/// A struct for handling HTTP requests. Takes care of the repetitive work of checking for errors, etc and exposes a simple interface
pub struct Client {
    inner: ClientWithMiddleware,
    ctx: Context,
}

impl Client {
    pub fn new(ctx: &Context) -> Self {
        Self {
            inner: ClientBuilder::new(reqwest::Client::new())
                .with(RetryTransientMiddleware::new_with_policy(
                    ExponentialBackoff::builder().build_with_max_retries(3),
                ))
                .build(),
            ctx: ctx.clone(),
        }
    }

    async fn request(
        &self,
        url: &str,
        method: RequestMethod,
        headers: &[(&str, Option<&str>)],
        ignore_401: bool,
    ) -> Result<Response, String> {
        let mut request = match method {
            RequestMethod::GET => self.inner.get(url),
            RequestMethod::HEAD => self.inner.head(url),
        };
        for (name, value) in headers {
            if let Some(v) = value {
                request = request.header(*name, *v)
            }
        }
        match request.send().await {
            Ok(response) => {
                let status = response.status();
                if status == 404 {
                    let message = format!("{} {}: Not found!", method, url);
                    self.ctx.logger.warn(&message);
                    Err(message)
                } else if status == 401 {
                    if ignore_401 {
                        Ok(response)
                    } else {
                        let message = format!("{} {}: Unauthorized! Please configure authentication for this registry or if you have already done so, please make sure it is correct.", method, url);
                        self.ctx.logger.warn(&message);
                        Err(message)
                    }
                } else if status == 502 {
                    let message = format!("{} {}: The registry is currently unavailabile (returned status code 502).", method, url);
                    self.ctx.logger.warn(&message);
                    Err(message)
                } else if status.as_u16() <= 400 {
                    Ok(response)
                } else {
                    match method {
                        RequestMethod::GET => error!(
                            "{} {}: Unexpected error: {}",
                            method,
                            url,
                            response.text().await.unwrap()
                        ),
                        RequestMethod::HEAD => error!(
                            "{} {}: Unexpected error: Recieved status code {}",
                            method, url, status
                        ),
                    }
                }
            }
            Err(error) => {
                if error.is_connect() {
                    let message = format!("{} {}: Connection failed!", method, url);
                    self.ctx.logger.warn(&message);
                    Err(message)
                } else if error.is_timeout() {
                    let message = format!("{} {}: Connection timed out!", method, url);
                    self.ctx.logger.warn(&message);
                    Err(message)
                } else if error.is_middleware() {
                    let message = format!("{} {}: Connection failed after 3 retries!", method, url);
                    self.ctx.logger.warn(&message);
                    Err(message)
                } else {
                    error!(
                        "{} {}: Unexpected error: {}",
                        method,
                        url,
                        error.to_string()
                    )
                }
            }
        }
    }

    pub async fn get(
        &self,
        url: &str,
        headers: &[(&str, Option<&str>)],
        ignore_401: bool,
    ) -> Result<Response, String> {
        self.request(url, RequestMethod::GET, headers, ignore_401)
            .await
    }

    pub async fn head(
        &self,
        url: &str,
        headers: &[(&str, Option<&str>)],
    ) -> Result<Response, String> {
        self.request(url, RequestMethod::HEAD, headers, false).await
    }
}
