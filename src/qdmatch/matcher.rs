
use crate::db::db_models::DocPerson;
use super::model::CandidatePerson;
use bson::{doc, Document};
use serde::{Deserialize, Serialize};
use super::rules::MatchRule;
use std::fs::File;
use std::io::Read;
use rand::thread_rng;
use rand::seq::SliceRandom;

const RULE_ID_AGE:i32 = 1;

//Macher has to be implemented by all Kind of Matchers
trait Matcher<T:PartialEq> {
    fn getList(&self) -> &Vec<T>;

    fn seeks(&self, value:&T) -> bool {
        self.calculate(value) > 0.0
    }

    fn calculate(&self, value:&T) -> f32 {
        let list = self.getList();
        let mut weight = 0.0;
        let count = list.len();
        if count <= 0 {
            return weight;
        }
       
        let least_count = 100.0 / (count as f32);
        
        for i in 0..count {
            if value == &list[i] {
                weight = least_count * ((count-i) as f32);
            }
        }

        weight
    }
}


pub enum MatchError {
    JsonError,
    NoRulesFound,
}

impl From<serde_json::error::Error> for MatchError {
    fn from(e:serde_json::error::Error) -> Self {
        MatchError::JsonError
    }
}



fn default_gender() -> String {
    "-".to_string()
}

fn default_1d_list() -> String {
    "1d".to_string()
}



pub struct MatcherFilter<T> {
    priority_list: Vec<T>
}

impl<T:Clone> MatcherFilter<T> {
    fn new(seeks:&Vec<T>) -> Option<Self> {
        let mut seeks_collection:Vec<T> = seeks.to_vec();
        Some(Self {
            priority_list:seeks_collection
        })
    }
}

impl<T:Clone+PartialEq> Matcher<T> for MatcherFilter<T> {
    fn getList(&self) ->&Vec<T> {
        &self.priority_list
    }
}




pub struct Match {
    person:DocPerson,
    age:MatcherFilter<f32>,
    gender:MatcherFilter<String>,
    education:MatcherFilter<String>,
    verbal:MatcherFilter<String>
}

impl Match {

    pub fn new(person:DocPerson, rule_file:String) -> Result<Option<Self>, MatchError>{
        //Read policy file and create matcher
        let mut file = File::open(rule_file).unwrap();
        let mut json:MatchRule = serde_json::from_reader(file).unwrap();

        let mut age:Option<MatcherFilter<f32>> = None;
        let mut gender:Option<MatcherFilter<String>> = None;
        let mut education:Option<MatcherFilter<String>> = None;
        let mut verbal:Option<MatcherFilter<String>> = None;
        for policy in json.policy {
            // TODO : Check for seeking as well
            if person.gender != policy.p_self {
                continue;
            }

            for rule_id in policy.p_rules {
                for rule in &json.rules {
                    //Not the rule we are looking for
                    if rule.rule_id != rule_id {
                        continue;
                    }

                    //We got the rule
                    match rule_id {

                        //Age Rule
                        1 => {
                            // Self deines the gender in Age Rule
                            let mut self_age = person.age.parse::<f32>().unwrap_or_default();
                            for node in &rule.rule_data {
                                if node.r_self.to_lowercase() == person.gender.to_lowercase() {
                                    let age_priority_list = (&node.r_seek_float).into_iter().map(|x| x + self_age).collect();
                                    age = MatcherFilter::new(&age_priority_list);
                                    break;
                                }
                            }
                        }

                        //Gender Rule
                        2 => {
                            //Sef define gender in Gender rule
                            for node in &rule.rule_data {
                                if node.r_self.to_lowercase() == person.gender.to_lowercase() {
                                    gender = MatcherFilter::new(&node.r_seek_string);
                                    break;
                                }
                            }

                        }

                        //Education
                        3 | 4 => {
                            for node in &rule.rule_data {
                                if node.r_self.to_lowercase() == person.education.to_lowercase() {
                                    education = MatcherFilter::new(&node.r_seek_string);
                                    break;
                                }
                            }

                            match education {
                                None => {
                                    education = MatcherFilter::new(&(Vec::new()));
                                }

                                _=> {}
                            }
                        }

                        //Verbal
                        5 => {
                            for node in &rule.rule_data {
                                if node.r_self.to_lowercase() == person.verbal_ability.to_lowercase() {
                                    verbal = MatcherFilter::new(&node.r_seek_string);
                                    break;
                                }
                            }

                            //Add Default list with zero element
                            match verbal {
                                None => {
                                    verbal = MatcherFilter::new(&(Vec::new()));
                                }

                                _=> {}
                            }
                        }

                        _ => {}
                    }
                }
            }

            //Load only one policy at a time
            break;
        }

        match (age, gender, education, verbal) {
            (Some(age), Some(gender), Some(education), Some(verbal)) => {
                return Ok(Some(Match {
                    person,
                    age,
                    gender,
                    education,
                    verbal
                }));
            }
            _ => {}
        }
        
        return Err(MatchError::NoRulesFound);

        
    }

    //Return the Query Which picks the sorted collection from database
    pub fn getFilter(&self) -> impl Into<Option<bson::ordered::OrderedDocument>> {   
        
        //Put the age and Gender filter as of now
        Some(doc! {"gender":{"$in":self.gender.getList()}})
    }

    //Calculate for cadidate person
    fn calculateScore(&self, candidate:&CandidatePerson) -> f32 {
        let mut weight = 0.0;
        weight += self.age.calculate(&candidate.age);
        weight += self.gender.calculate(&candidate.gender);
        weight += self.verbal.calculate(&candidate.verbal);
        weight += self.education.calculate(&candidate.education);
        weight/4.0
    }

    //Function which take can candidate like and return sorted list
    pub fn qurate(&mut self, candidates:Vec<CandidatePerson>) -> Vec<CandidatePerson> {
        
        let mut sortList:Vec<CandidatePerson> = Vec::new();

        //Filter


        //Calculate Score
        for c in candidates {
            let mut candidate = c;
            candidate.match_score = self.calculateScore(&candidate);
            sortList.push(candidate);
            
        }
        sortList.shuffle(&mut thread_rng());
        sortList.sort_by(|a, b| b.cmp_score(a));
        sortList
    }
}

