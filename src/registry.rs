use std::{fs, path::PathBuf};

use once_cell::sync::Lazy;
use semver::Version;
use toml::{Table, Value};
use url::Url;

use crate::{error::Error, Result};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Registry {
    CratesIo { version: Version }, // todo: check version requirements
    Git { repo: Url },
    Else { registry: Url, version: Version },
}

static USERNAME: Lazy<String> =
    Lazy::new(|| username::get_user_name().expect("Can't get name of the current user..."));

impl Registry {
    pub fn parse_from_value(value: &Value) -> Result<Self> {
        match value {
            Value::String(ref raw_version) => {
                let version = Version::parse(raw_version).map_err(|_| Error::InvalidVersion)?;
                Ok(Registry::CratesIo { version })
            }
            Value::Table(t) => Registry::parse_from_table(t),
            _ => todo!("check under which conditions this may occur..."),
        }
    }
    pub fn version(&self) -> Option<Version> {
        match self {
            Self::CratesIo { version, .. } | Self::Else { version, .. } => Some(version.clone()),
            Self::Git { .. } => None,
        }
    }
    pub fn path(&self) -> PathBuf {
        assert!(
            matches!(self, Registry::CratesIo { .. }),
            "Only crates from crates.io are supported!"
        );

        let username = Lazy::force(&USERNAME); // initialize the cell once
        let src_path = PathBuf::from(format!("/home/{username}/.cargo/registry/src"));

        let index = fs::read_dir(src_path)
            .expect("No source registry directory")
            .filter_map(|p| p.ok())
            .map(|p| p.path())
            .find(|p| {
                let filename = p.file_name().unwrap();
                filename.to_str().unwrap().starts_with("index.crates.io")
            })
            .expect("No crates.io index");

        index
    }
    fn parse_from_table(table: &Table) -> Result<Self> {
        if let Some(Value::Boolean(t)) = table.get("optional") {
            if *t {
                return Err(Error::OptionalDependency); // todo: these are also tricky, ignoring them for now
            }
        }

        if let Some(Value::String(raw_version)) = table.get("version") {
            let version = Version::parse(raw_version).map_err(|_| Error::InvalidVersion)?;

            if let Some(Value::String(raw_registry)) = table.get("registry") {
                let registry = Url::parse(raw_registry).map_err(|_| Error::InvalidRegistryUrl)?;
                Ok(Registry::Else { registry, version })
            } else {
                Ok(Registry::CratesIo { version })
            }
        } else if let Some(Value::String(raw_repo)) = table.get("git") {
            let repo = Url::parse(raw_repo).map_err(|_| Error::InvalidGitRepoUrl)?;
            Ok(Registry::Git { repo })
        } else {
            todo!("proper error type: invalid toml config")
        }
    }
}
