use crate::{
    constants::{SERVER, SERVICE, STORAGE},
    pkg::Pkg,
};
use std::{error::Error, io::Cursor};
use wac_graph::{CompositionGraph, EncodeOptions, NodeId, types::Package};
use wasm_pkg_client::{Client, PublishOpts};

mod constants;
mod get;
mod pkg;

trait ErrInto<T> {
    fn err_into(self) -> Result<T, Box<dyn Error>>;
}

impl<T, E: Into<Box<dyn Error>>> ErrInto<T> for Result<T, E> {
    fn err_into(self) -> Result<T, Box<dyn Error>> {
        self.map_err(Into::into)
    }
}

fn instantiate(
    graph: &mut CompositionGraph,
    pkg: Pkg,
    bytes: Vec<u8>,
) -> Result<NodeId, Box<dyn Error>> {
    let v = pkg.version.parse()?;
    Package::from_bytes(pkg.name, Some(&v), bytes, graph.types_mut())
        .err_into()
        .and_then(|package| graph.register_package(package).err_into())
        .map(|package| graph.instantiate(package))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let client = Client::with_global_defaults().await?;
    let (storage_bytes, service_bytes) =
        tokio::try_join!(get::get(&client, STORAGE), get::get(&client, SERVICE))?;
    tracing::log::info!("Downloaded storage and service packages");
    let mut graph = CompositionGraph::new();

    let storage_instance = instantiate(&mut graph, constants::STORAGE, storage_bytes)?;
    let service_instance = instantiate(&mut graph, constants::SERVICE, service_bytes)?;

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

    let (package_ref, version) = client
        .publish_release_data(
            Box::pin(cursor),
            PublishOpts {
                package: Some((SERVER.package_name().parse()?, SERVER.version.parse()?)),
                registry: None,
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
