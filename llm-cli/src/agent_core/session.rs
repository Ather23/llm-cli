use uuid::Uuid;

pub struct Session {
    pub id: String,
}

impl Session {
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        Session { id }
    }
}
