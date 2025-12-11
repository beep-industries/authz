use crate::{authzed::api::v1::ObjectReference, infrastructure::common::authzed::entities::Id};

pub struct Server(Id);

impl Into<ObjectReference> for Server {
    fn into(self) -> ObjectReference {
        ObjectReference {
            object_type: "server".to_string(),
            object_id: self.0,
        }
    }
}

impl From<String> for Server {
    fn from(id: String) -> Self {
        Server(Id::from(id))
    }
}
