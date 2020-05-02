use std::fmt;
use serde::Deserialize;
use crate::qdmatch::model::CandidatePerson;


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
        write!(f, "{:<10} {:<40} {:<5} {:<10} {:<15} {:<15}", 
            self.qid, self.name, self.age, self.gender, self.education, self.verbal_ability)
    }
}





#[derive(Debug, Deserialize)]
pub struct CandidatePersonDb {
    pub qid:String,

    #[serde(default = "default_string")]
    pub name:String,

    #[serde(default = "default_string")]
    pub gender:String,

    #[serde(default = "default_string")]
    pub age:String,

    #[serde(default = "default_string")]
    pub phone:String,

    #[serde(default = "default_string")]
    pub email:String,

    #[serde(default = "default_string")]
    pub education:String,

    #[serde(default = "default_string")]
    pub verbal_ability:String,
}

impl Into<CandidatePerson> for CandidatePersonDb {
    fn into(self) -> CandidatePerson {
        CandidatePerson {
            qid:self.qid,
            name:self.name,
            age:self.age.parse::<f32>().unwrap_or_default(),
            gender:self.gender,
            education:self.education,
            verbal:self.verbal_ability,
            phone:self.phone,
            email:self.email,
            match_score:0.0
        }
    }
}



//Config Files
#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub app:String,
    pub username:String,
    pub password:String,
    pub source:String,
    pub database:String,
    pub host:String,
    pub port:String,
}