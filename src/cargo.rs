use serde::Serialize;
use toml::value::Table;

use crate::errors::CargoPlayError;

#[derive(Clone, Debug, Serialize)]
struct CargoPackage {
    name: String,
    version: String,
    edition: String,
}

impl CargoPackage {
    fn new(name: String) -> Self {
        Self {
            name: name.to_lowercase(),
            version: "0.1.0".into(),
            edition: "2018".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CargoManifest {
    package: CargoPackage,
    dependencies: Table,
}

impl CargoManifest {
    pub(crate) fn new(name: String, dependencies: Vec<String>) -> Result<Self, CargoPlayError> {
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
            package: CargoPackage::new(name),
            dependencies,
        })
    }
}
