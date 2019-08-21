use std::collections::HashSet;

use serde::Serialize;
use toml::value::{Table, Value};

use crate::errors::CargoPlayError;
use crate::opt::RustEdition;

#[derive(Clone, Debug, Serialize)]
struct CargoPackage {
    name: String,
    version: String,
    edition: String,
}

impl CargoPackage {
    fn new(name: String, edition: RustEdition) -> Self {
        Self {
            name: name.to_lowercase(),
            version: "0.1.0".into(),
            edition: edition.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CargoManifest {
    package: CargoPackage,
    #[serde(serialize_with = "toml::ser::tables_last")]
    dependencies: Table,
}

impl CargoManifest {
    pub(crate) fn new(
        name: String,
        dependencies: Vec<String>,
        edition: RustEdition,
    ) -> Result<Self, CargoPlayError> {
        let dependencies = dependencies
            .into_iter()
            .map(|dependency| dependency.parse::<toml::Value>())
            .collect::<Result<Vec<toml::Value>, _>>()
            .map_err(CargoPlayError::from_serde)?;

        if dependencies.iter().any(|d| !d.is_table()) {
            return Err(CargoPlayError::ParseError("format error!".into()));
        }

        let dependencies: Table = dependencies
            .into_iter()
            .map(|d| d.try_into::<Table>().unwrap().into_iter())
            .flatten()
            .collect();

        Ok(Self {
            package: CargoPackage::new(name, edition),
            dependencies,
        })
    }

    fn normalize_crate_name(name: &str) -> String {
        name.replace("-", "_")
    }

    fn normalized_dependencies(&self) -> HashSet<String> {
        self.dependencies
            .clone()
            .into_iter()
            .map(|(key, _)| Self::normalize_crate_name(&key))
            .collect()
    }

    pub(crate) fn add_infers(&mut self, infers: HashSet<String>) {
        let existing = self.normalized_dependencies();

        // we don't need to normalize crate name here (in filter) since it's impossible to have
        // dash in use statments.
        self.dependencies.extend(
            infers
                .into_iter()
                .filter(|key| !existing.contains(key))
                .map(|key| (key, Value::String("*".into()))),
        );
    }
}
