use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use url::Url;

use crate::{binaries::Binaries, resources::Resources};

pub mod builder;

pub use builder::Builder;

pub struct Version {
  version_base: Url,
  eveclient_path: String,
  binaries_base: Url,
  resources_base: Url,
  client: reqwest_middleware::ClientWithMiddleware,
}

impl Version {
  pub async fn new(
    version_base: Url,
    eveclient_path: String,
    binaries_base: Url,
    resources_base: Url,
    client: reqwest_middleware::ClientWithMiddleware,
  ) -> Self {
    Self {
      version_base,
      eveclient_path,
      binaries_base,
      resources_base,
      client,
    }
  }

  pub async fn build(&self) -> Result<String> {
    let res = self
      .client
      .get(
        self
          .version_base
          .join(self.eveclient_path.as_str())
          .context("make eveclient url")?,
      )
      .send()
      .await
      .context("get eveclient")?;
    let eveclient = res.text().await?;
    let eveclient = serde_json::from_str::<EveClient>(eveclient.as_str())?;
    Ok(eveclient.build)
  }

  pub async fn binaries(&self) -> Result<Binaries> {
    let build = self.build().await?;

    let binaries_index = self
      .version_base
      .join(format!("eveonline_{build}.txt").as_str())?;
    let binaries_index = self
      .client
      .get(binaries_index)
      .send()
      .await?
      .bytes()
      .await?;
    let mut binaries_index = csv::ReaderBuilder::new()
      .has_headers(false)
      .from_reader(binaries_index.iter().as_slice());
    let binaries_index = binaries_index
      .deserialize::<BinariesIndexRecord>()
      .map(|x| match x {
        Ok(BinariesIndexRecord { url, path, .. }) => Ok((Url::parse(url.as_str())?, path)),
        Err(err) => Err(anyhow::Error::new(err)),
      })
      .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(Binaries::new(
      self.binaries_base.to_owned(),
      binaries_index,
      self.client.to_owned(),
    ))
  }

  pub async fn resources(&self) -> Result<Resources> {
    let binaries = self.binaries().await?;
    let Some(resources_index) = binaries.get(Url::parse("app:/resfileindex.txt")?).await? else {
      return Err(anyhow!("no 'app:/resfileindex.txt' in binaries_index"));
    };
    let mut reader = StreamReader::new(resources_index);
    let mut resources_index = Vec::new();
    reader.read_to_end(&mut resources_index).await?;
    let mut resources_index = csv::ReaderBuilder::new()
      .has_headers(false)
      .from_reader(resources_index.as_slice());
    let resources_index = resources_index
      .deserialize::<ResourcesIndexRecord>()
      .map(|x| match x {
        Ok(ResourcesIndexRecord { url, path, .. }) => Ok((Url::parse(url.as_str())?, path)),
        Err(err) => Err(anyhow::Error::new(err)),
      })
      .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(Resources::new(
      self.resources_base.to_owned(),
      resources_index,
      self.client.to_owned(),
    ))
  }
}

#[derive(Debug, Deserialize)]
struct EveClient {
  pub build: String,
  #[allow(dead_code)]
  pub protected: bool,
  #[allow(dead_code)]
  pub platforms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BinariesIndexRecord {
  url: String,
  path: String,
  #[allow(dead_code)]
  v3: String,
  #[allow(dead_code)]
  v4: String,
  #[allow(dead_code)]
  v5: String,
  #[allow(dead_code)]
  v6: String,
}

#[derive(Debug, Deserialize)]
struct ResourcesIndexRecord {
  url: String,
  path: String,
  #[allow(dead_code)]
  v3: String,
  #[allow(dead_code)]
  v4: String,
  #[allow(dead_code)]
  v5: String,
}
