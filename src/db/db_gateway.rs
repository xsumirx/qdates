#![crate_name = "doc"] 

//  DbGateway
//  This is a wrapper class which can hook to databases
//  like MongoDb, MySQL etc.


use mongodb::{Client, options::ClientOptions, Database};
use bson::{doc, Bson};
use mongodb::{options::{FindOneOptions}};
use super::db_models::DocPerson;
use crate::qdmatch::model::CandidatePerson;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct CandidatePersonDb {
    pub qid:String,
    pub name:String,
    pub gender:String,
    pub age:String,
    pub education:String,
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
            match_score:0.0
        }
    }
}


const DEFAULT_COLLECTION_PERSON:&'static str = "persons";

/// A database wraper which provides neccessary operation specific for the application
pub struct DbGateway {
    /// Path of the database server 
    uri:String,

    /// MongoDB database references
    client:Option<Client>,
    database:Option<Database>
}


/// Database errors reported by this module
#[derive(Debug)]
pub enum DbError {
    //NetworkError,
    //QueryError,
    MongoError,
    ReconnetRequestError,
    UnwrapError,
    NoPersonFound,
    DbParsingError,
}

impl std::convert::From<mongodb::error::Error> for DbError {
    fn from(m_error:mongodb::error::Error) -> Self {
        println!("Mongo Error :: {:?}", m_error);
        DbError::MongoError
    }
}


impl DbGateway {

    /// Returns a new database, which is ready to connect with specified uri
    /// 
    /// # Argument
    /// 
    /// * `uri` - Uri of the database server
    /// 
    pub fn new(uri:&str) -> Self {
        DbGateway {
            uri:uri.to_string(),
            client:None,
            database:None,
        }
    }

    /// Connect to the datbase, Before performing any datbase related
    /// operation this function needs to be called once and only once at the begining of the program.
    pub fn connect(&mut self) -> Result<(),DbError> {

        //Check if client is already initialized
        match &self.client {
            //Client never been initialized.
            None => {
                println!("Connecting to database ....");
            }
            
            //We already have a client.
            _ => {
                println!("Database already connected !");
                return Err(DbError::ReconnetRequestError)
            }
        }

        //Connect to the client
        let mut client_options = ClientOptions::parse(self.uri.as_str())?;
        client_options.app_name = Some("QDates".to_string());
        let client = Client::with_options(client_options)?;
        
        //Select the database
        let database = client.database("qdates");
        
        println!("Database Connected !");

        //Move Ownerships and return
        self.database = Some(database);
        self.client = Some(client);
        Ok(())
    }

    /// Get a person from the database by searching through its uiq
    pub fn getPerson(&mut self, qid:&String) -> Result<Option<DocPerson>, DbError> {

        if let Some(db) = &self.database {
            //Get Person
            let filter = doc! { "qid":qid};
            let collection = db.collection(DEFAULT_COLLECTION_PERSON);
            let find_options = FindOneOptions::builder()
                .sort(doc!{"name":1})
                .build();
            
            if let Ok(_doc) = collection.find_one(filter, find_options) {
                if let Some(document) = _doc {
                    match bson::from_bson::<DocPerson>(bson::Bson::Document(document)) {
                        Ok(person) => {
                            return Ok(Some(person));
                        }
                        Err(e) => {
                            return Err(DbError::DbParsingError);
                        }
                    }
                    
                }else {
                    return Err(DbError::NoPersonFound);
                }
            }
        }

        Err(DbError::MongoError)
    }


    pub fn getCandidates(&mut self, filter: impl Into<Option<bson::ordered::OrderedDocument>>) 
        -> Result<Vec<CandidatePerson>, DbError> {
            
        let mut persons:Vec<CandidatePerson> = Vec::new();
        if let Some(db) = &self.database {
            let collection = db.collection(DEFAULT_COLLECTION_PERSON);
            //let mut filter:Document = doc! { "qid":{"$ne":"1"}};
            let mut cursor = collection.find(filter, None)?;
            for result in cursor {
                match result {
                    Ok(document) => {
                        match bson::from_bson::<CandidatePersonDb>(bson::Bson::Document(document)) {
                            Ok(personDb) => {
                                let person:CandidatePerson = personDb.into();
                                persons.push(person);
                            }
                            Err(e) => {
                                println!("{:?}", e);
                            }
                        }
                    }
                    
                    Err(e) => {
                        println!("Error while iterating in db cursor");
                    }
                }
            }

            return Ok(persons);
        }

        Err(DbError::MongoError)
    }

    //Update Person
    pub fn insert_check_duplicates(&mut self, persons:Vec<DocPerson>) -> Result<Vec<DocPerson>, DbError> {

        let mut failed_entries:Vec<DocPerson> = Vec::new();

        if let Some(db) = &self.database {
            let collection = db.collection(DEFAULT_COLLECTION_PERSON);
            for person in persons {
                //let filter = doc! { "qid": &person.qid };
                let filter = doc! { "qid": &person.qid, };
                let find_options = FindOneOptions::builder()
                    .sort(doc!{"name":1})
                    .build();
                if let Ok(_doc) = collection.find_one(filter, find_options) {
                    if let Some(_) = _doc {
                        //Entry Already Present
                        failed_entries.push(person);
                    }else{
                        //No Such Entry, Insert It
                        let doc_to_insert = doc! { 
                            "qid": &person.qid, 
                            "name": &person.name,
                            "gender":&person.gender,
                            "age":&person.age.to_string(),
                            "email":&person.email,
                            "phone":&person.phone,
                            "city":&person.city,
                            "languages":&person.languages,
                            "education":&person.education,
                            "response_rating":&person.response_rating.to_string(),
                            "verbal_ability":&person.verbal_ability,
                            "seeking":&person.seeking,

                        };
                        if let Err(_) = collection.insert_one(doc_to_insert, None) {
                            failed_entries.push(person);
                        }

                    }
                }else{
                    //Problem with the database
                    failed_entries.push(person);
                }
            }
        }
        

        Ok(failed_entries)
    }

}