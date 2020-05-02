use std::fmt;
use std::cmp::Ordering;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct CandidatePerson {
    pub qid:String,
    pub name:String,
    pub gender:String,
    pub age:f32,
    pub education:String,
    pub verbal:String,
    pub email:String,
    pub phone:String,

    pub match_score:f32
}

impl CandidatePerson {
    pub fn cmp_score(&self, other:&Self) -> Ordering {
        if self.match_score > other.match_score {
            return Ordering::Greater;
        }else if self.match_score < other.match_score {
            return Ordering::Less;
        }

        Ordering::Equal
    }

    pub fn print_detail(&self) {
        println!("{:<10} {:<40} {:<5} {:<10} {:<15} {:<15} {:<15} {:<20}", 
        self.qid, self.name, self.age, self.gender, self.education, self.verbal, self.phone, self.email);
    }
}

impl fmt::Display for CandidatePerson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<10} {:<40} {:<5} {:<10} {:<15} {:<15} {:15}%", 
            self.qid, self.name, self.age, self.gender, self.education, self.verbal,  (self.match_score as u8))
    }
}