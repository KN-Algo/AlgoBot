use serenity::all::CreateInteractionResponseMessage;

pub trait IntoResponse {
    fn into_response(&self) -> CreateInteractionResponseMessage;
}

impl IntoResponse for &str {
    fn into_response(&self) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new().content(*self)
    }
}

impl IntoResponse for String {
    fn into_response(&self) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new().content(self)
    }
}
