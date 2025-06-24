use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use http_cache_reqwest::{CACacheManager, Cache, HttpCache};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use url::Url;

use super::Version;

pub struct Builder {
  version_base: Url,
  eveclient_path: String,
  binaries_base: Url,
  resources_base: Url,
  client: reqwest::ClientBuilder,
  reqwest_middlewares: Vec<Arc<dyn reqwest_middleware::Middleware>>,
  retry_times: Option<u32>,
  cache_dir: Option<PathBuf>,
}

impl Builder {
  pub fn infinity() -> Self {
    Self {
      version_base: Url::parse("https://eve-china-version-files.oss-cn-hangzhou.aliyuncs.com/")
        .expect("incorrect infinity url"),
      eveclient_path: "/eveclient_INFINITY.json".to_string(),
      binaries_base: Url::parse("https://ma79.gdl.netease.com/eve/binaries/")
        .expect("incorrect infinity url"),
      resources_base: Url::parse("https://ma79.gdl.netease.com/eve/resources/")
        .expect("incorrect infinity url"),
      client: reqwest::ClientBuilder::new(),
      reqwest_middlewares: Vec::new(),
      retry_times: None,
      cache_dir: None,
    }
  }

  pub fn tranquility() -> Self {
    Self {
      version_base: Url::parse("https://binaries.eveonline.com/").expect("incorrect infinity url"),
      eveclient_path: "/eveclient_TQ.json".to_string(),
      binaries_base: Url::parse("https://binaries.eveonline.com/").expect("incorrect infinity url"),
      resources_base: Url::parse("https://resources.eveonline.com/")
        .expect("incorrect infinity url"),
      client: reqwest::ClientBuilder::new(),
      reqwest_middlewares: Vec::new(),
      retry_times: None,
      cache_dir: None,
    }
  }

  pub fn build(mut self) -> Result<Version> {
    if let Some(retry_times) = self.retry_times {
      self
        .reqwest_middlewares
        .push(Arc::new(RetryTransientMiddleware::new_with_policy(
          ExponentialBackoff::builder().build_with_max_retries(retry_times),
        )));
    }
    if let Some(cache_dir) = self.cache_dir {
      self.reqwest_middlewares.push(Arc::new(Cache(HttpCache {
        mode: http_cache_reqwest::CacheMode::Default,
        manager: CACacheManager { path: cache_dir },
        options: http_cache_reqwest::HttpCacheOptions::default(),
      })))
    }
    self
      .reqwest_middlewares
      .push(Arc::new(TracingMiddleware::default()));

    Ok(Version {
      version_base: self.version_base,
      eveclient_path: self.eveclient_path,
      binaries_base: self.binaries_base,
      resources_base: self.resources_base,
      client: reqwest_middleware::ClientWithMiddleware::new(
        self.client.build()?,
        self.reqwest_middlewares,
      ),
    })
  }

  pub fn with(&mut self, middleware: impl reqwest_middleware::Middleware) -> &mut Self {
    self.reqwest_middlewares.push(Arc::new(middleware));
    self
  }

  pub fn retry(&mut self, times: u32) -> &mut Self {
    self.retry_times = Some(times);
    self
  }

  pub fn cache_to(&mut self, path: PathBuf) -> &mut Self {
    self.cache_dir = Some(path);
    self
  }
}
