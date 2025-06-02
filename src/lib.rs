pub mod binaries;
pub mod resources;
pub mod sde_client;
pub mod version;

pub use binaries::Binaries;
pub use resources::Resources;
pub use sde_client::SdeClient;
pub use version::Version;

#[cfg(test)]
mod test {
  use tracing::level_filters::LevelFilter;

  use crate::{SdeClient, version};

  #[tokio::test]
  async fn get_build() {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
      .compact()
      .with_max_level(LevelFilter::DEBUG)
      .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).unwrap();

    SdeClient::from_version(version::Builder::infinity().build().unwrap())
      .await
      .unwrap()
      .build()
      .await
      .unwrap();
  }
}
