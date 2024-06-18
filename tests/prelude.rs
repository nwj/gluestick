#[derive(Debug)]
pub enum Error {}

impl<Err: std::fmt::Display> From<Err> for Error {
    // This will cause Error to panic whenever ? runs on it.
    // This is useful in tests, where we want to panic early
    // but also want the concision of the ? syntactic sugar.
    #[track_caller]
    fn from(err: Err) -> Self {
        panic!("error: {}: {}", std::any::type_name::<Err>(), err);
    }
}

pub type Result<T> = core::result::Result<T, Error>;
