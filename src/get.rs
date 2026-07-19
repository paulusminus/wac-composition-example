use crate::pkg::Pkg;
use anyhow::Error;
use futures_util::TryStreamExt;
use wasm_pkg_client::Client;

pub async fn get(client: &Client, pkg: Pkg) -> Result<Vec<u8>, Error> {
    let package = pkg.package_name().parse()?;
    let version = pkg.version.parse()?;
    let release = client.get_release(&package, &version).await?;
    let mut bytes = Vec::<u8>::new();
    let mut stream = client.stream_content(&package, &release).await?;
    while let Some(chunk) = stream.try_next().await? {
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
