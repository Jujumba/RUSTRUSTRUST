use std::{
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};

use toml::{Table, Value};

use crate::{error::Error, registry::Registry, Result};

// Orig files come from Cargo: it obfuscates the original `Cargo.toml` file, copying the original
// to `Cargo.toml.orig`
static IGNORED_EXTENSIONS: [&'static str; 10] = [
    "md", "toml", "lock", "json", "sh", "orig", "txt", "pdf", "odt", "sql",
];
// DO NOT DARE TO HIDE C CODE THERE!!!!
static IGNORED_DIRS: [&'static str; 5] = ["target", "benches", "tests", "examples", "migrations"];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dependency {
    name: String,
    registry: Registry,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DependencyRoot(PathBuf);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceFile(PathBuf);

impl Dependency {
    pub fn parse<A>(toml: &A) -> Result<Vec<Self>>
    where
        A: AsRef<str>,
    {
        let toml = toml.as_ref();
        let table: Table = toml.parse().map_err(|_| Error::InvalidToml)?;

        let Some(Value::Table(dependencies)) = table.get("dependencies") else {
            return Ok(Vec::new()); // if there are no dependencies
        };

        let mut result = Vec::new(); // todo: capacity = dependencies.keys().count() (?)

        for (name, value) in dependencies
            .keys()
            .map(Clone::clone) // owned keys are needed
            .zip(dependencies.values())
        {
            let registry = match Registry::parse_from_value(value) {
                Ok(registry) => registry,
                Err(Error::OptionalDependency) => continue,
                Err(err) => return Err(err), // type system enforces, so no @ bindings 😭
            };
            result.push(Dependency { name, registry })
        }

        Ok(result)
    }
    pub fn get_sources(self) -> Vec<SourceFile> {
        let root: DependencyRoot = self.into();
        root.into()
    }
}
impl SourceFile {
    pub fn new<I>(path: I) -> Result<Self>
    where
        I: Into<PathBuf>,
    {
        let path = path.into();
        Self::inner_new(path)
    }
    // Behold...
    #[allow(non_snake_case)]
    pub fn THE_TEST(&self) -> Result<()> {
        let content = fs::read_to_string(&self.0).unwrap(); // todo: handle the error

        if syn::parse_file(&content).is_ok() {
            Ok(())
        } else {
            Err(Error::IMPURITIES)
        }
    }
    fn inner_new(path: PathBuf) -> Result<Self> {
        if path.is_dir() {
            return Err(Error::IMPURITIES); // I do like the name `IMPURITIES`.
        }
        let Some(filename) = path.file_name().and_then(|osstr| osstr.to_str()) else {
            return Err(Error::IMPURITIES); // todo: But I have to return something else :)
        };
        if filename.starts_with('.') {
            return Err(Error::IMPURITIES);
        }
        let Some(extension) = path.extension().and_then(|osstr| osstr.to_str()) else {
            return Err(Error::IMPURITIES);
        };
        if IGNORED_EXTENSIONS.contains(&extension) {
            return Err(Error::IMPURITIES); // todo: And here too...
        }
        Ok(SourceFile(path))
    }
}
impl DependencyRoot {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        if path.is_dir() {
            Ok(DependencyRoot(path))
        } else {
            Err(Error::NotCrateRoot)
        }
    }
}
impl From<DependencyRoot> for Vec<SourceFile> {
    fn from(root: DependencyRoot) -> Self {
        let mut stack = vec![root];
        let mut files = Vec::new();
        while let Some(entry) = stack.pop() {
            /* Todo:
             * Sometimes crate is not downloaded and other version is used
             * (even if the precise one is specified).
             * I have 0 ideas why and don't want to spend
             * few hours figuring out how cargo resolves that stuff, thus ignoring for now.
             *
             * Perhaps the crate is stored somewhere else, but this is tricky in anyway
             */
            let Ok(entries) = fs::read_dir(entry) else {
                continue;
            };
            for path in entries
                .into_iter()
                .filter_map(io::Result::ok)
                .map(|entry| entry.path())
            {
                if path.is_dir() {
                    // todo: handle it
                    let dirname = path.file_name().and_then(|os| os.to_str()).unwrap();
                    if !IGNORED_DIRS.contains(&dirname) && !dirname.starts_with('.') {
                        stack.push(DependencyRoot(path));
                    }
                    continue;
                }
                let Ok(file) = SourceFile::new(path) else {
                    // if this file ignored or whatever
                    continue;
                };
                files.push(file);
            }
        }
        files
    }
}
impl From<Dependency> for Vec<SourceFile> {
    fn from(dependency: Dependency) -> Self {
        let source: DependencyRoot = dependency.into();
        source.into()
    }
}
impl From<Dependency> for DependencyRoot {
    fn from(mut dependency: Dependency) -> DependencyRoot {
        let mut path: PathBuf = dependency.registry.path();
        dependency.name.push('-');

        let version = dependency.registry.version().unwrap().to_string();
        // Safety: always sound, as `version` is a UTF-8 encoded string
        unsafe {
            dependency
                .name
                .as_mut_vec()
                .append(&mut version.into_bytes());
        }

        path.push(dependency.name);

        DependencyRoot(path)
    }
}
impl TryFrom<PathBuf> for DependencyRoot {
    type Error = Error;
    fn try_from(path: PathBuf) -> Result<Self> {
        Self::new(path)
    }
}
impl Deref for DependencyRoot {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Path> for DependencyRoot {
    fn as_ref(&self) -> &Path {
        self.deref()
    }
}
