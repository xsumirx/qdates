use std::fmt;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct DocPerson {
    pub qid:String,                 // A unique id in the system
    pub name:String,
    pub gender:String,
    pub age:String,                     
    pub email:String,               // Primary Mail address
    pub phone:String,               // Phone No. including country code
    pub city:String,                // City the person is living in
    pub languages:Vec<String>,       // What language can this person speaks in priority order

    #[serde(default = "default_string")]
    pub profession:String,          // What does this person do for living

    #[serde(default = "default_string")]
    pub education:String,           // Education of the Person

    #[serde(default = "default_string")]
    pub response_rating:String,         // How well this person reponds outof 10

    #[serde(default = "default_string")]
    pub verbal_ability:String,

    #[serde(default = "default_string")]
    pub seeking:String,

    //So
}

fn default_string() -> String {
    " ".to_string()
}

impl fmt::Display for DocPerson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<5} {:<20} {:<5} {:<10} {:15} {:<30}", self.qid, self.name, self.age, self.gender, self.phone, self.email)
    }
}