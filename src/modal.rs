use serenity::{
    all::{Context, InteractionId},
    async_trait,
};

pub use crate::response::ModalResponse;
pub use modal_macro::Modal;

#[async_trait]
pub trait Modal
where
    Self: Sized,
{
    async fn execute(
        ctx: &Context,
        id: InteractionId,
        token: &str,
    ) -> serenity::Result<(Self, ModalResponse)>;
}

//
//pub struct ModalResponse {
//    fields: HashMap<String, String>,
//    pub interaction_id: InteractionId,
//    pub token: String,
//}
//
//pub struct Modal {
//    title: String,
//    fields: Vec<Field>,
//    timeout: Duration,
//}
//
//
//    //bad clones
//    pub async fn execute(
//        self,
//        ctx: &Context,
//        interaction_id: InteractionId,
//        token: &str,
//    ) -> serenity::Result<Option<ModalResponse>> {
//        let mut modal = CreateQuickModal::new(self.title).timeout(self.timeout);
//        let mut map: HashMap<String, String> = HashMap::new();
//        for field in &self.fields {
//            modal = match field.field_type {
//                FieldType::Short => modal.short_field(field.name.clone()),
//                FieldType::Paragraph => modal.paragraph_field(field.name.clone()),
//            };
//        }
//        let response = match modal.execute(ctx, interaction_id, token).await? {
//            Some(r) => r,
//            None => return Ok(None),
//        };
//
//        for (i, field) in response.inputs.iter().enumerate() {
//            map.insert(self.fields[i].name.clone(), field.clone());
//        }
//
//        Ok(Some(ModalResponse {
//            fields: map,
//            interaction_id: response.interaction.id,
//            token: response.interaction.token,
//        }))
//    }
//}
