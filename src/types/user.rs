use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct User{
   pub public_key:String,
   pub name:String
}