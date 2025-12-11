use crate::{
    authzed::api::v1::{ObjectReference, SubjectReference},
    infrastructure::common::authzed::entities::{Entity, Id},
};

pub struct User(Id);

impl Into<ObjectReference> for User {
    fn into(self) -> ObjectReference {
        ObjectReference {
            object_type: Entity::User.into(),
            object_id: self.0,
        }
    }
}

impl Into<SubjectReference> for User {
    fn into(self) -> SubjectReference {
        SubjectReference {
            object: Some(self.into()),
            ..Default::default()
        }
    }
}

impl From<String> for User {
    fn from(value: String) -> Self {
        User(Id::from(value))
    }
}