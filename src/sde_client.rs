use crate::{Binaries, Resources, Version};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use futures::{StreamExt, stream::BoxStream};
use url::Url;

pub struct SdeClient {
  version: Version,
  binaries: Binaries,
  resources: Resources,
}

impl SdeClient {
  pub fn new(version: Version, binaries: Binaries, resources: Resources) -> Self {
    Self {
      version,
      binaries,
      resources,
    }
  }

  pub async fn from_version(version: Version) -> Result<Self> {
    Ok(Self {
      binaries: version.binaries().await?,
      resources: version.resources().await?,
      version,
    })
  }

  pub async fn build(&self) -> Result<String> {
    self.version.build().await
  }

  pub async fn get(&self, url: Url) -> Result<Option<BoxStream<Result<Bytes, std::io::Error>>>> {
    match url.scheme() {
      "app" => self.binaries.get(url).await.map(|x| x.map(|x| x.boxed())),
      "res" => self.resources.get(url).await.map(|x| x.map(|x| x.boxed())),
      _ => Err(anyhow!("do not support this scheme")),
    }
  }
}
