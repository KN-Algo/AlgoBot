use serenity::all::{CommandInteraction, InteractionId};

pub struct ModalResponse {
    pub id: InteractionId,
    pub token: String,
}

pub struct Response {
    pub msg: String,
    pub interaction_id: InteractionId,
    pub token: String,
}

impl Response {
    pub fn from_command(command: &CommandInteraction, msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            interaction_id: command.id,
            token: command.token.clone(),
        }
    }

    pub fn from_modal(modal_response: ModalResponse, msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            interaction_id: modal_response.id,
            token: modal_response.token,
        }
    }
}
