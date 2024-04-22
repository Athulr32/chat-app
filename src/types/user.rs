use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct User{
    public_key:String,
    name:String
}