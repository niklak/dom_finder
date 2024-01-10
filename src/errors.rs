use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("the required `{0}` field is missing")]
    FieldIsMissing(String),
    #[error("it is only possible to use either 'extract' or 'children' options")]
    ExtractOrDive,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("matcher can be empty only if inherit is set to true")]
    RequireMatcher,
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Pipeline(#[from] PipelineError),
}

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error("pipeline proc with name `{0}` does not exist")]
    ProcDoesNotExist(String),
    #[error("pipeline proc `{0}`: not enough arguments, require {1}, got {2}")]
    ProcNotEnoughArguments(String, usize, usize),
}
