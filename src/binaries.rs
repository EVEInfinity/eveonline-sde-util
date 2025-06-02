use std::collections::HashMap;

use anyhow::Result;
use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use tracing::debug;
use url::Url;

pub struct Binaries {
  binaries_base: Url,
  index: HashMap<Url, String>,
  client: reqwest_middleware::ClientWithMiddleware,
}

impl Binaries {
  pub fn new(
    binaries_base: Url,
    index: HashMap<Url, String>,
    client: reqwest_middleware::ClientWithMiddleware,
  ) -> Self {
    Self {
      binaries_base,
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
    debug!("{}", self.binaries_base.join(path)?);
    Ok(Some(
      self
        .client
        .get(self.binaries_base.join(path)?)
        .send()
        .await?
        .bytes_stream()
        .map_err(std::io::Error::other),
    ))
  }
}
