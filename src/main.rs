use crate::{
    constants::{SERVICE, STORAGE},
    pkg::Pkg,
};
use std::error::Error;
use wac_graph::{CompositionGraph, EncodeOptions, NodeId, types::Package};

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

async fn instantiate(
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
    let (storage_bytes, service_bytes) = tokio::try_join!(get::get(STORAGE), get::get(SERVICE))?;
    let mut graph = CompositionGraph::new();

    let storage_instance = instantiate(&mut graph, constants::STORAGE, storage_bytes).await?;
    let service_instance = instantiate(&mut graph, constants::SERVICE, service_bytes).await?;

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
    tokio::fs::write(constants::OUTPUT_FILE, &bytes).await?;
    Ok(())
}
