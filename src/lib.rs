#![allow(non_snake_case)]

mod dependency;
mod error;
mod registry;

use dependency::SourceFile;
use dependency::{Dependency, DependencyRoot};
use error::Error;

type Result<T> = std::result::Result<T, Error>;

/// # Panics:
/// If the crate or its dependencies are impure (that is, not 💯% **`RUST`**)
pub fn THE_TEST() {
    walk(DependencyRoot::new(".").unwrap());
}
fn walk(crate_root: DependencyRoot) {
    let Ok(content) = std::fs::read_to_string(crate_root.join("Cargo.toml")) else {
        return;
    };
    let Ok(dependencies) = Dependency::parse(&content) else {
        return;
    };
    let dependencies: Vec<DependencyRoot> = dependencies.into_iter().map(Into::into).collect();
    for dependency in dependencies {
        let sources: Vec<SourceFile> = dependency.clone().into();
        for source in sources {
            if source.THE_TEST().is_err() {
                panic!("IMPURITY {source:?} is found in {dependency:?}")
            }
        }
        walk(dependency);
    }
}
