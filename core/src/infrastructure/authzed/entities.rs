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
