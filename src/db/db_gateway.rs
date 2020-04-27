//  DbGateway
//  This is a wrapper class which can hook to databases
//  like MongoDb, MySQL etc.


use mongodb::{Client, options::ClientOptions, Database};
use bson::{doc};
use mongodb::options::{FindOneOptions};
use super::db_models::DocPerson;


const DEFAULT_COLLECTION_PERSON:&'static str = "persons";

pub struct DbGateway {
    uri:String,
    client:Option<Client>,
    database:Option<Database>
}


#[derive(Debug)]
pub enum DbError {
    //NetworkError,
    //QueryError,
    MongoError,
    ReconnetRequestError,
    UnwrapError,
}

impl std::convert::From<mongodb::error::Error> for DbError {
    fn from(m_error:mongodb::error::Error) -> Self {
        println!("Mongo Error :: {:?}", m_error);
        DbError::MongoError
    }
}



impl DbGateway {

    //New DbGateway instance
    pub fn new(uri:&str) -> Self {
        DbGateway {
            uri:uri.to_string(),
            client:None,
            database:None,
        }
    }

    //Connect to Database
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


    //Update Person
    pub fn insert_check_duplicates(&mut self, persons:Vec<DocPerson>) -> Result<Vec<DocPerson>, DbError> {

        let mut failed_entries:Vec<DocPerson> = Vec::new();

        if let Some(db) = &self.database {
            let collection = db.collection(DEFAULT_COLLECTION_PERSON);
            for person in persons {
                //let filter = doc! { "qid": &person.qid };
                let filter = doc! { "qid": &person.qid, "gender":{"$in":["male", "female", "Female"]}};
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