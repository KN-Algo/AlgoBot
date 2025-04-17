use serenity::all::{
    CommandInteraction, Context, InteractionId, ModalInteraction, PartialChannel, User,
};

use crate::modal::Modal;

pub struct Response {
    pub id: InteractionId,
    pub token: String,
    pub msg: String,
}

pub trait Respond {
    fn respond(self, msg: impl Into<String>) -> serenity::Result<Response>;
}

pub struct Interaction<'ctx> {
    ctx: &'ctx Context,
    pub id: InteractionId,
    pub token: String,
    pub name: String,
    pub user: User,
    pub channel: Option<PartialChannel>,
}

impl<'ctx> Interaction<'ctx> {
    pub fn from_command(ctx: &'ctx Context, cmd: &CommandInteraction) -> Self {
        Self {
            ctx,
            id: cmd.id,
            token: cmd.token.clone(),
            name: cmd.data.name.clone(),
            user: cmd.user.clone(),
            channel: cmd.channel.clone(),
        }
    }

    pub fn from_modal(ctx: &'ctx Context, modal: ModalInteraction) -> Self {
        Self {
            ctx,
            id: modal.id,
            token: modal.token,
            name: "".to_owned(),
            user: modal.user,
            channel: modal.channel,
        }
    }

    pub async fn modal<T: Modal<'ctx>>(self) -> serenity::Result<T> {
        let modal = T::execute(self.ctx, self.id, &self.token).await?;
        Ok(modal)
    }
}

impl Respond for Interaction<'_> {
    fn respond(self, msg: impl Into<String>) -> serenity::Result<Response> {
        Ok(Response {
            id: self.id,
            token: self.token.clone(),
            msg: msg.into(),
        })
    }
}
