use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Error;
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand};
use wapm_toml::{Bindings, Wapm};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetadataTable {
    pub wapm: Wapm,
}

#[tracing::instrument(skip_all)]
pub(crate) fn parse_cargo_toml(
    manifest_path: Option<&Path>,
    no_default_features: bool,
    features: Option<&Features>,
    all_features: bool,
) -> Result<Metadata, Error> {
    let mut cmd = MetadataCommand::new();

    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }

    if let Ok(path) = std::env::current_dir() {
        cmd.current_dir(path);
    }

    if no_default_features {
        cmd.features(CargoOpt::NoDefaultFeatures);
    }

    if let Some(features) = features {
        cmd.features(CargoOpt::SomeFeatures(features.0.clone()));
    }

    if all_features {
        cmd.features(CargoOpt::AllFeatures);
    }

    tracing::debug!(cmd = ?cmd.cargo_command(), "Parsing Cargo metadata");

    let meta = cmd.exec()?;

    Ok(meta)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Features(pub Vec<String>);

impl From<&'_ str> for Features {
    fn from(value: &'_ str) -> Self {
        Features(value.split(',').map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use wapm_toml::{WaiBindings, WitBindings};

    use super::*;

    #[test]
    fn parse_wai_bindings() {
        let table = toml::toml! {
            [wapm]
            namespace = "wasmer"
            abi = "none"
            bindings = { wai-version = "0.1.0", exports = "hello-world.wai" }
        };
        let should_be = MetadataTable {
            wapm: Wapm {
                namespace: "wasmer".to_string(),
                package: None,
                wasmer_extra_flags: None,
                abi: wapm_toml::Abi::None,
                fs: None,
                bindings: Some(Bindings::Wai(WaiBindings {
                    exports: Some("hello-world.wai".into()),
                    imports: Vec::new(),
                    wai_version: "0.1.0".parse().unwrap(),
                })),
            },
        };

        let got = MetadataTable::deserialize(table).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_wit_bindings() {
        let table = toml::toml! {
            [wapm]
            namespace = "wasmer"
            abi = "none"
            bindings = { wit-bindgen = "0.1.0", wit-exports = "hello-world.wit" }
        };
        let should_be = MetadataTable {
            wapm: Wapm {
                namespace: "wasmer".to_string(),
                package: None,
                wasmer_extra_flags: None,
                abi: wapm_toml::Abi::None,
                fs: None,
                bindings: Some(Bindings::Wit(WitBindings {
                    wit_bindgen: "0.1.0".parse().unwrap(),
                    wit_exports: "hello-world.wit".into(),
                })),
            },
        };

        let got = MetadataTable::deserialize(table).unwrap();

        assert_eq!(got, should_be);
    }
}
