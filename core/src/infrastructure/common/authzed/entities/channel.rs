use crate::{authzed::api::v1::ObjectReference, infrastructure::common::authzed::entities::Id};

pub struct Channel(Id);

impl Into<ObjectReference> for Channel {
    fn into(self) -> ObjectReference {
        ObjectReference {
            object_type: "channel".to_string(),
            object_id: self.0,
        }
    }
}

impl From<String> for Channel {
    fn from(id: String) -> Self {
        Channel(Id::from(id))
    }
}
