#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lua error: {0}")]
    LuaError(#[from] luaur::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error)
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Self::LuaError(e) => Self::LuaError(e.clone()),
            Self::IoError(e) => Self::IoError(std::io::Error::new(e.kind(), e.to_string()))
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;