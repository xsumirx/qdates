//  DbGateway
//  This is a wrapper class which can hook to databases
//  like MongoDb, MySQL etc.

use std::fmt;
use std::fs::File;
use bson::{doc, Bson};
use std::iter::{IntoIterator, Iterator};
use mongodb::{
    Client,
    Database,
    options::{
        auth::{
            Credential,
            AuthMechanism
        }, 
    StreamAddress, 
    ClientOptions, 
    FindOneOptions, 
    FindOneAndUpdateOptions
    }
};
use mongodb::options::IndexModel;
use super::db_models::{DocPerson, CandidatePersonDb, DbConfig};
use crate::qdmatch::model::CandidatePerson;
use serde::Deserialize;



struct DbIndexModels {
    iter:Vec<IndexModel>,
}

struct DbIndexModelIterator {
    iter:Vec<IndexModel>,
}

impl Iterator for DbIndexModelIterator {

    type Item = IndexModel;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.pop()
    }
}


impl IntoIterator for DbIndexModels {
    type Item = IndexModel;
    type IntoIter = DbIndexModelIterator;

    fn into_iter(self) -> Self::IntoIter {
        DbIndexModelIterator {
            iter:self.iter
        }
    }
}

impl DbIndexModels {
    fn new() -> Self {
        Self {
            iter:Vec::new()
        }
    }

    fn add(&mut self, node:IndexModel) {
        self.iter.push(node);
    }
}




const DEFAULT_COLLECTION_PERSON:&'static str = "persons";

/// A database wraper which provides neccessary operation specific for the application
pub struct DbGateway {
    /// Database Settings
    config:DbConfig,

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

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::NoPersonFound => {
                write!(f, "No person found !")
            }

            _ => {
                write!(f, "Database Error Occured")
            }
        }
        
    }
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
    pub fn new(filename:&str) -> Option<Self> {

        //Read policy file and create matcher
        if let Ok(file) = File::open(filename) {
            if let Ok(config) = serde_json::from_reader(file) {
                return Some(DbGateway {
                    config:config,
                    client:None,
                    database:None,
                });
            }
        }
        
        //No Config file found
        None

        
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
        let auth_mechnism = AuthMechanism::ScramSha256;
        let cred = Credential::builder()
            .username(String::from(&self.config.username))
            .password(String::from(&self.config.password))
            .source(Some(String::from(&self.config.source)))
            .mechanism(Some(auth_mechnism))
            .build();

        

        let port = self.config.port.parse::<u16>();
        let mut portNo:Option<u16>= None;
        if let Ok(_port) = port {
            portNo = Some(_port);
        }

        let client_options = ClientOptions::builder()
            .hosts(vec!{StreamAddress{
                hostname:String::from(&self.config.host),
                port:portNo
            }})
            .direct_connection(true)
            .app_name(String::from(&self.config.app))
            .credential(Some(cred))
            .build();

        let client = Client::with_options(client_options)?;
        
        //Select the database
        let database = client.database(&self.config.database[..]);

        if let Err(e) = database.list_collections(None, None) {
            println!("{:?}", e);
            return Err(DbError::MongoError);
        }

        
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
            
            match collection.find_one(filter, find_options) {
                Ok(_doc) => {
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

                Err(e) => {
                    println!("{:?}", e);
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
    pub fn insertAndCheckDuplicate(&mut self, persons:Vec<DocPerson>, checkDuplicate:bool) 
        -> Result<Vec<DocPerson>, DbError> {

        let mut failed_entries:Vec<DocPerson> = Vec::new();

        if let Some(db) = &self.database {
            
            for person in persons {
                //let filter = doc! { "qid": &person.qid };
                let collection = db.collection(DEFAULT_COLLECTION_PERSON);
                let filter = doc! { "qid": &person.qid };
                let updateOption = FindOneAndUpdateOptions::builder()
                    .upsert(false)
                    .build();
                    
                if checkDuplicate == false {
                    let doc_to_update = doc! { 
                        "$set":{
                            "qid":&person.qid,
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
                        }
                    };
                    match  collection.find_one_and_update(filter, doc_to_update, updateOption) {
                        Ok(Some(d)) => {}
                        Err(_) | Ok(None) => {
                            failed_entries.push(person);
                        }
                        
                    }

                    continue;
                }

                if let Ok(_doc) = collection.find_one(filter, None) {
                    if let Some(doc) = _doc {
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