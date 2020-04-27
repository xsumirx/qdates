use std::fmt;


pub struct DocPerson {
    pub qid:String,                 // A unique id in the system
    pub name:String,
    pub gender:String,
    pub age:u8,                     
    pub email:String,               // Primary Mail address
    pub phone:String,               // Phone No. including country code
    pub city:String,                // City the person is living in
    pub languages:Vec<String>,       // What language can this person speaks in priority order
    pub profession:String,          // What does this person do for living
    pub education:String,           // Education of the Person
    pub response_rating:u8,         // How well this person reponds outof 10
    pub verbal_ability:String,
    pub seeking:String,

    //So
}

impl fmt::Display for DocPerson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<5} {:<20} {:<5} {:<10} {:15} {:<30}", self.qid, self.name, self.age, self.gender, self.phone, self.email)
    }
}