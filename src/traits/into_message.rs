use serenity::all::CreateMessage;

pub trait IntoMessage {
    fn into_msg(&self) -> CreateMessage;
}

impl IntoMessage for &str {
    fn into_msg(&self) -> CreateMessage {
        CreateMessage::new().content(*self)
    }
}

impl IntoMessage for String {
    fn into_msg(&self) -> CreateMessage {
        CreateMessage::new().content(self)
    }
}
