#[derive(Debug)]
pub enum Error {
    InvalidVersion,
    InvalidToml,
    OptionalDependency,
    InvalidGitRepoUrl,
    InvalidRegistryUrl,
    NotCrateRoot,
    IMPURITIES,
}
