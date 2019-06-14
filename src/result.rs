use failure::{Backtrace,Fail,Context};
use std::io;
use std::fmt::{self,Display};

#[derive(Fail, Debug)]
pub enum KvsErrorKind {
    #[fail(display = "Input was invalid UTF-8 at index {}", _0)]
    Utf8Error(usize),

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    ParserError(#[cause] serde_json::error::Error),

    #[fail(display = "Not Found: {}", _0)]
    NotFound(String),
}

#[derive(Debug)]
pub struct KvsError {
    inner: Context<KvsErrorKind>,
}

impl KvsError {
    pub fn kind<'a>(&'a self) -> &'a KvsErrorKind {
        self.inner.get_context()
    }
}

impl From<KvsErrorKind> for KvsError {
    fn from(kind: KvsErrorKind) -> KvsError {
        KvsError { inner: Context::new(kind) }
    }
}

impl From<Context<KvsErrorKind>> for KvsError {
    fn from(inner: Context<KvsErrorKind>) -> KvsError {
        KvsError { inner: inner }
    }
}

impl Fail for KvsError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

pub type Result<T> = std::result::Result<T,KvsError>;