use std::fmt::Display;

#[derive(Debug)]
pub enum BotError {
    Serenity(serenity::Error),
    Db(sqlx::Error),
    ChronoParse(chrono::ParseError),
}

impl From<serenity::Error> for BotError {
    fn from(value: serenity::Error) -> Self {
        Self::Serenity(value)
    }
}

impl From<sqlx::Error> for BotError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(value)
    }
}

impl From<chrono::ParseError> for BotError {
    fn from(value: chrono::ParseError) -> Self {
        Self::ChronoParse(value)
    }
}

impl From<sqlx::migrate::MigrateError> for BotError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::Db(value.into())
    }
}

impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Serenity(e) => format!("serenity: {e}"),
            Self::Db(e) => format!("database: {e}"),
            Self::ChronoParse(e) => format!("chrono: {e}"),
        };

        write!(f, "{}", s)
    }
}
