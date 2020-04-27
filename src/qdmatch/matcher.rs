use crate::db::db_models::DocPerson;
use super::model::CandidatePerson;
use mongodb::{Client, options::ClientOptions, Database};
use bson::{doc, Document};

//Macher has to be implemented by all Kind of Matchers
trait Matcher<T:PartialEq> {
    fn get_list(&self) -> &Vec<T>;
    fn calculate(&self, value:T) -> f32 {
        let list = self.get_list();
        let count = list.len();
        let mut weight = 0.0;
        let least_count = 100.0 / (count as f32);
        for i in 0..count-1 {
            if value == list[i] {
                weight = least_count * ((count - i) as f32);
            }
        }

        weight
    }
}




// Gender Matcher
pub struct MatcherGender {
    filter_type:String,
    priority_list:Vec<String>
}

impl MatcherGender {
    fn new(_type:String, _list:Vec<String>) -> Self {
        MatcherGender {
            filter_type:_type,
            priority_list:_list
        }
    }
}

impl Matcher<String> for MatcherGender {
    fn get_list(&self) ->&Vec<String> {
        &self.priority_list
    }
}




//Age Matcher
pub struct MatcherAge {
    //All Sort of matching rule variable
    filter_type:String,
    priority_list:Vec<i8>
}

impl MatcherAge {
    fn new(_type:String, _list:Vec<i8>) -> Self {
        MatcherAge {
            filter_type:_type,
            priority_list:_list
        }
    }
}

impl Matcher<i8> for MatcherAge {
    fn get_list(&self) ->&Vec<i8> {
        &self.priority_list
    }
}





pub struct Match {
    person:DocPerson,
    age:MatcherAge,
    gender:MatcherGender,
}

impl Match {

    fn new(person:DocPerson, rule_file:String){
        //Read policy file and create matcher
        
    }

    //Return the Query Which picks the sorted collection from database
    fn make_db_query(&self) -> Option<Document> {
        Some(doc! { "qid": &self.person.qid, "gender":{"$in":["male", "female", "Female"]}})
    }

    //Calculate for cadidate person
    fn calculate_score(&self, candidate:&CandidatePerson) -> f32 {
        let mut weight = 0.0;
        weight += self.age.calculate(candidate.age);
        weight += self.gender.calculate(String::from(&candidate.gender));
        weight/2.0
    }

    //Function which take can candidate like and return sorted list
    fn qurate(&mut self) -> Vec<CandidatePerson> {


        Vec::new()
    }
}

