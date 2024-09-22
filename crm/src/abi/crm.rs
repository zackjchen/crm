use crate::pb::crm::User;
use prost_types::Timestamp;
use std::time::SystemTime;
impl User {
    pub fn new(id: i32, name: &str, email: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
            created_at: Some(Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }),
        }
    }
}
