use crate::{
    constants::{SERVER, SERVICE, STORAGE},
    get::get,
    pkg::Pkg,
};
use anyhow::Error;
use std::io::Cursor;
use wac_graph::{CompositionGraph, EncodeOptions, NodeId, types::Package};
use wasm_pkg_client::{Client, Config, PublishOpts};

mod constants;
mod get;
mod oci;
mod pkg;

trait ErrInto<T> {
    fn err_into(self) -> Result<T, Error>;
}

impl<T, E: Into<Error>> ErrInto<T> for Result<T, E> {
    fn err_into(self) -> Result<T, Error> {
        self.map_err(Into::into)
    }
}

fn instantiate(graph: &mut CompositionGraph, pkg: Pkg, bytes: Vec<u8>) -> Result<NodeId, Error> {
    let v = pkg.version.parse()?;
    Package::from_bytes(pkg.name, Some(&v), bytes, graph.types_mut())
        .err_into()
        .and_then(|package| graph.register_package(package).err_into())
        .map(|package| graph.instantiate(package))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    tracing::log::info!("Starting");
    let config = Config::from_toml(include_str!("../wasm-pkg.toml"))?;
    let client = Client::new(config);
    let (storage_bytes, service_bytes) =
        tokio::try_join!(get(&client, STORAGE), get(&client, SERVICE))?;
    tracing::log::info!("Downloaded storage and service packages");

    let mut graph = CompositionGraph::new();

    let storage_instance = instantiate(&mut graph, constants::STORAGE, storage_bytes)?;
    let service_instance = instantiate(&mut graph, constants::SERVICE, service_bytes)?;

    for i in graph.packages() {
        tracing::log::info!("Package: {:?}", i);
    }

    // Alias the default export of the `pm:lipl-storage-fs` instance
    let export_types =
        graph.alias_instance_export(storage_instance, constants::STORAGE_INTERFACE)?;

    // Set argument `b` of the instantiation of `my:package2` to `a`
    graph.set_instantiation_argument(
        service_instance,
        constants::STORAGE_INTERFACE,
        export_types,
    )?;

    let export_handler =
        graph.alias_instance_export(service_instance, constants::SERVICE_INTERFACE)?;
    graph.export(export_handler, constants::SERVICE_INTERFACE)?;

    // Finally, encode the graph into a new component
    let bytes = graph.encode(EncodeOptions::default())?;
    tracing::log::info!("Encoded graph into component");

    let cursor = Cursor::new(bytes);
    // let namespace_label = Label::from_str(SERVER.namespace)?;
    // let name_label = Label::from_str(SERVER.name)?;
    // let package_ref = PackageRef::new(namespace_label, name_label);
    // let version = Version::from_str(SERVER.version)?;
    // let registry = Registry::from_str("ghcr.io")?;

    // let oci_client_config = ClientConfig::default();
    // let oci_client = wasm_pkg_client::oci::client::Client::new(oci_client_config);
    // let oci_reference = Reference::from_str("ghcr.io/paulusminus/pm/lipl-server:0.2.1")?;
    // oci_client.push_blob(&oci_reference, bytes, digest).await?;

    let (package_ref, version) = client
        .publish_release_data(
            Box::pin(cursor),
            PublishOpts {
                package: Some((SERVER.package_name().parse()?, SERVER.version.parse()?)),
                ..Default::default()
            },
        )
        .await?;
    tracing::log::info!(
        "Package: {}:{}@{} published",
        package_ref.namespace(),
        package_ref.name(),
        version
    );
    Ok(())
}
