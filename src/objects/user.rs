use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::actor::PersonType,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use chrono::{Local, NaiveDateTime};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    AppData,
    activities::create_post::CreatePost,
    entities::{
        prelude::*,
        user::Model as DbUser
    },
    error::AppError,
};

// ActivityPub 用户
// ActivityPub Person
// https://github.com/LemmyNet/activitypub-federation-rust/blob/main/docs/03_federating_users.md
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    // 用户 ID，应为域名 + 用户名（运行时生成）
    id: ObjectId<DbUser>,
    // 类型，应始终为 Person
    #[serde(rename = "type")]
    kind: PersonType,
    // 用户名（应为域名）
    // `example.com`
    name: String,
    // 首选用户名（应为网站标题）
    // `Example Domain`
    preferred_username: String,
    // 用户描述
    // summary: Option<String>,
    // 用户头像
    // icon: Option<PersonImage>,
    // 用户背景图
    // image: Option<PersonImage>,
    // 收件箱
    // `https://hatsu.local/example.com/inbox`
    inbox: Url,
    // 发件箱
    // `https://hatsu.local/example.com/outbox`
    outbox: Url,
    // 公钥
    public_key: PublicKey,
    // ActivityPub 用户附件（Metadata）
    // ActivityPub Person Attachment (Metadata)
    // attachment: Vec<PersonAttachment>,
    // 关注者
    // followers: Url,
    // 正在关注
    // following: Url,
}

// ActivityPub 用户图像
// ActivityPub Person Image
// #[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PersonImage {
//     // 类型，应始终为 Image
//     #[serde(rename = "type")]
//     kind: String,
//     // 图片链接
//     url: String,
// }

// ActivityPub 用户附件（Metadata）
// ActivityPub Person Attachment (Metadata)
// #[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PersonAttachment {
//     // 类型，应始终为 PropertyValue
//     #[serde(rename = "type")]
//     kind: String,
//     name: String,
//     value: String,
// }

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    CreateNote(CreatePost)
}

// 数据库用户 Feed
// Database User Feed
// #[derive(Clone, Debug)]
// pub struct DbUserFeed {
//     // json / atom / rss
//     name: String,
//     value: Url,
// }

#[async_trait::async_trait]
impl Object for DbUser {
    type DataType = AppData;
    type Kind = Person;
    type Error = AppError;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(NaiveDateTime::parse_from_str(&self.last_refreshed_at, "%Y-%m-%d %H:%M:%S").unwrap())
    }

    // 从 ID 读取
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(User::find_by_id(&object_id.to_string())
            .one(&data.conn)
            .await?)
    }

    // 转换为 ActivityStreams JSON
    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(Person {
            name: self.name.clone(),
            preferred_username: self.preferred_username.clone(),
            kind: Default::default(),
            id: Url::parse(&self.id).unwrap().into(),
            inbox: Url::parse(&self.inbox)?,
            outbox: Url::parse(&self.outbox)?,
            public_key: self.public_key(),
        })
    }

    // 验证
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        Ok(())
    }

    // 转换为本地格式
    async fn from_json(
        json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: json.id.to_string(),
            name: json.name,
            preferred_username: json.preferred_username,
            inbox: json.inbox.to_string(),
            outbox: json.outbox.to_string(),
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Local::now().naive_local().format("%Y-%m-%d %H:%M:%S").to_string(),
            // followers: vec![],
            local: false,
        })
    }

    // 删除用户
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let _delete_user = User::delete_by_id(&self.id.to_string())
            .exec(&data.conn)
            .await?;
        Ok(())
    }
}

impl Actor for DbUser {
    fn id(&self) -> Url {
        Url::parse(&self.id).unwrap()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
    }

    fn inbox(&self) -> Url {
        Url::parse(&self.inbox).unwrap()
    }
}
