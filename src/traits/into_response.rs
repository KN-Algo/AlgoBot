use serenity::all::CreateInteractionResponseMessage;

pub trait IntoResponse {
    fn into_msg(&self) -> CreateInteractionResponseMessage;
}

impl IntoResponse for &str {
    fn into_msg(&self) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new().content(*self)
    }
}
