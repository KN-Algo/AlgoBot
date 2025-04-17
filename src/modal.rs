use serenity::{
    all::{Context, InteractionId},
    async_trait,
};

pub use modal_macro::Modal;

#[async_trait]
pub trait Modal<'ctx>
where
    Self: Sized,
{
    async fn execute(ctx: &'ctx Context, id: InteractionId, token: &str) -> serenity::Result<Self>;
}

#[macro_export]
macro_rules! modal {
    ($struct_name:ident($modal_name:literal, $modal_time:literal) { $($field_name:ident => $field_type:ident($field_title:literal), )+ }) => {
        #[derive($crate::modal::Modal)]
        #[modal($modal_name, $modal_time)]
        struct $struct_name<'ctx> {
            interaction: crate::response::Interaction<'ctx>,
            $(
                #[$field_type($field_title)]
                $field_name: String,
            )+

        }

        impl<'ctx> crate::response::Respond for $struct_name<'ctx> {
            fn respond(self, msg: impl Into<String>) -> ::serenity::Result<crate::response::Response> {
                Ok(crate::response::Response { id: self.interaction.id, token: self.interaction.token.clone(), msg: msg.into() })
            }
        }
    };
}
