use crate::authzed::api::v1::{Relationship, RelationshipUpdate};

pub enum Operation {
    Create,
    Delete,
    Touch,
}

impl Into<i32> for Operation {
    /// FYI https://buf.build/authzed/api/file/main:authzed/api/v1/core.proto#L157
    fn into(self) -> i32 {
        match self {
            Operation::Create => 1,
            Operation::Touch => 2,
            Operation::Delete => 3,
        }
    }
}


impl Into<RelationshipUpdate> for Relationship {
    fn into(self) -> RelationshipUpdate {
        RelationshipUpdate {
            operation: 0,
            relationship: Some(self),
        }
    }
}

pub trait Action {
    fn delete(&self) -> RelationshipUpdate;
    fn create(&self) -> RelationshipUpdate;
    fn touch(&self) -> RelationshipUpdate;
}

impl<T: Into<Relationship> + Clone> Action for T {
    fn delete(&self) -> RelationshipUpdate {
        let mut relationship_update: RelationshipUpdate = Into::<Relationship>::into(self.clone()).into();
        relationship_update.operation = Operation::Delete.into(); 
        relationship_update
    }

    fn create(&self) -> RelationshipUpdate {
        let mut relationship_update: RelationshipUpdate = Into::<Relationship>::into(self.clone()).into();
        relationship_update.operation = Operation::Delete.into(); 
        relationship_update
    }

    fn touch(&self) -> RelationshipUpdate {
        let mut relationship_update: RelationshipUpdate = Into::<Relationship>::into(self.clone()).into();
        relationship_update.operation = Operation::Touch.into(); 
        relationship_update
    }
}
