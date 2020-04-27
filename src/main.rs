pub mod excel;
pub mod qdmatch;
pub mod db;


use clap::{Arg, App, SubCommand};
use db::{db_gateway};

fn main() {

    

    //Command line parser
    let matches = App::new("QDates")
        .version("0.0")
        .author("Sumir Kr. Jha <sumirkumarjha@gmail.com>")
        .about("QDates profile matcher")
        .subcommand(SubCommand::with_name("update")
            .about("Update database from excel file")
            .version("0.0")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input excel file to use")
                .required(true)
                .index(1)))
        .get_matches();

    

    //Check if Excel update is requested
    if let Some(matches) = matches.subcommand_matches("update") {
        let filename = matches.value_of("INPUT").unwrap();

        // Read the data from the file
        match excel::read(filename.to_string()) {
            Ok((person_collection, warning_collection)) => {
                //We have got the person and those field as well which doesn't feels like person
                
                //Print warning first
                for warning in warning_collection {
                    println!("{}", warning);
                }

                //Create Database
                let mut db = db_gateway::DbGateway::new("mongodb://localhost:27017");

                //Connect Database
                if let Err(e) = db.connect() {
                    println!("{:?}", e);
                    return;
                }else {
                    match db.insert_check_duplicates(person_collection) {
                        Ok(db_result) =>  {
                            //Unsucessfull entries
                            if db_result.len() > 0 {
                                println!("- Following entires failed - [Duplicate Entries]");
                                for person in db_result {
                                    println!("{}", person);
                                }
                            }
                            
                        }

                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                   
                }

                
            }
            
            Err(e) => {
                println!("{:?}", e);
            }    
        }
    }



    
    
}
