use serde::{Serialize,Deserialize};



#[derive(Serialize,Deserialize)]
pub struct Users{
    public_key:String,
    name:String
}