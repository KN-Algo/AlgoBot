use std::fmt::Display;

#[derive(Debug)]
pub enum BotError {}

impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error")
    }
}
