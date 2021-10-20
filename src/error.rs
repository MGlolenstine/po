use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error{
    #[snafu(display("Path doesn't exist: {}", path))]
    PathDoesNotExist{path: String},
}
