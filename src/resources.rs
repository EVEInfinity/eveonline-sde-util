use std::collections::HashMap;

use anyhow::Result;
use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use url::Url;

pub struct Resources {
  resources_base: Url,
  index: HashMap<Url, String>,
  client: reqwest_middleware::ClientWithMiddleware,
}

impl Resources {
  pub fn new(
    resources_base: Url,
    index: HashMap<Url, String>,
    client: reqwest_middleware::ClientWithMiddleware,
  ) -> Self {
    Self {
      resources_base,
      index,
      client,
    }
  }

  pub async fn get(
    &self,
    url: Url,
  ) -> Result<Option<impl Stream<Item = Result<Bytes, std::io::Error>>>> {
    let Some(path) = self.index.get(&url) else {
      return Ok(None);
    };
    Ok(Some(
      self
        .client
        .get(self.resources_base.join(path)?)
        .send()
        .await?
        .bytes_stream()
        .map_err(std::io::Error::other),
    ))
  }
}
