use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct User {
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub org_id: u32,
    pub roles: Vec<String>,
}
