use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::public,
    protocol::{
        helpers::deserialize_one_or_many,
        context::WithContext
    },
    traits::{ActivityHandler, Object},
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    AppData,
    AppError,
    entities::{
        prelude::*,
        user::Model as DbUser,
        post::Model as DbPost,
    },
    protocol::{
        activities::CreateOrUpdateType,
        objects::Note
    },
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateNote {
    pub(crate) actor: ObjectId<DbUser>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: Note,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) cc: Vec<Url>,
    #[serde(rename = "type")]
    pub(crate) kind: CreateOrUpdateType,
    pub(crate) id: Url,
}

impl CreateOrUpdateNote {
    pub async fn send(
        user_id: ObjectId<DbUser>,
        post_id: ObjectId<DbPost>,
        content: String,
        kind: CreateOrUpdateType,
        data: Data<AppData>
    ) -> Result<(), AppError> {
        let user: DbUser = User::find_by_id(
            format!("https://{}/u/{}", data.domain(), user_id)
        )
            .one(&data.conn)
            .await?
            .unwrap();
        
        let note = Note {
            kind: Default::default(),
            id: post_id,
            attributed_to: Url::parse(&user.id)?.into(),
            to: vec![public()],
            // TODO: cc: followers
            cc: vec![Url::parse(&format!("https://{}/u/{}/followers", data.domain(), user_id))?],
            content,
            in_reply_to: None,
            tag: vec![],
        };

        let create_or_update_note = Self {
            id: Url::parse(&format!("https://{}/o/{}", data.domain(), Uuid::new_v4()))?,
            actor: note.attributed_to.clone(),
            to: note.to.clone(),
            cc: note.cc.clone(),
            object: note,
            kind
        };

        let create_or_update_note_with_context = WithContext::new_default(create_or_update_note);

        // TODO: save note & create_or_update_note to database

        user.send(create_or_update_note_with_context, vec![public()], &data).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateNote {
    type DataType = AppData;
    type Error = AppError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // TODO
        DbPost::verify(&self.object, &self.id, data).await?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // TODO
        DbPost::from_json(self.object, data).await?;
        Ok(())
    }
}
