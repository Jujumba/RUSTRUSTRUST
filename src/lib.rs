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
fn check_sources(sources: Vec<SourceFile>, dependency: &DependencyRoot) {
    for source in sources {
        if source.THE_TEST().is_err() {
            panic!("IMPURITY {source:?} is found in {dependency:?}")
        }
    }
}
fn walk(crate_root: DependencyRoot) {
    check_sources(crate_root.clone().get_sources(), &crate_root);
    let Ok(toml) = std::fs::read_to_string(crate_root.join("Cargo.toml")) else {
        return;
    };
    let Ok(dependencies) = Dependency::parse_from_toml(&toml) else {
        return;
    };
    let dependencies: Vec<DependencyRoot> = dependencies.into_iter().map(Into::into).collect();
    for dependency in dependencies {
        let sources: Vec<SourceFile> = dependency.clone().get_sources();
        check_sources(sources, &dependency);
        walk(dependency);
    }
}
