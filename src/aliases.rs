use crate::error::BotError;

pub type TypedResult<T> = std::result::Result<T, BotError>;
pub type Result = std::result::Result<(), BotError>;
