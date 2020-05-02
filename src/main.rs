pub mod excel;
pub mod db;
pub mod qdmatch;


use clap::{Arg, App, SubCommand};
use db::{db_gateway};
use qdmatch::matcher::Match;
use bson::{doc};

fn main(){

    //Command line parser
    let matches = App::new("QDates")
        .version("0.0")
        .author("Sumir Kr. Jha <sumirkumarjha@gmail.com>")
        .about("QDates profile matcher")
        .subcommand(SubCommand::with_name("insert")
            .about("Insert into database from excel file")
            .version("0.0")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input excel file to use")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("update")
            .about("Update database from excel file")
            .version("0.0")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input excel file to use")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("search")
            .about("Search for people by name")
            .version("0.0")
            .arg(Arg::with_name("NAME")
                .help("Name of the person to search for")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("match")
            .about("Match person")
            .version("0.0")
            .arg(Arg::with_name("QID")
                .help("QID of the person for whom the date is to be qurated.")
                .required(true)
                .index(1)))
        .get_matches();


        
    //Create Database
    let mut db;
    if let Some(_db) = db_gateway::DbGateway::new("config.json"){
        db = _db;
    }else{
        println!("Bad config file !");
        return;
    }

    //Connect Database
    if let Err(e) = db.connect() {
        println!("{:?}", e);
        return;
    }

    //Check if Excel update is requested
    match matches.subcommand_name() {

        Some("update") => {
            let filename = matches.subcommand_matches("update").unwrap().value_of("INPUT").unwrap();
            // Read the data from the file
            // Read the data from the file
            match excel::read(filename.to_string()) {
                Ok((person_collection, warning_collection)) => {
                    //We have got the person and those field as well which doesn't feels like person
                    
                    //Print warning first
                    if warning_collection.len() > 0 {
                        println!("Following column couldn't be updated");
                    }

                    for warning in warning_collection {
                        println!("{}", warning);
                    }

                    
                    match db.insertAndCheckDuplicate(person_collection, false) {
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
                
                Err(e) => {
                    println!("{:?}", e);
                }    
            }
        }

        Some("search") => {
            let nameToSearch = matches.subcommand_matches("search").unwrap().value_of("NAME").unwrap();

            //Prepare the filter to match
            let filter =    doc! {
                                    "$or": [
                                                {"name": { "$regex": &nameToSearch, "$options": "i" }},
                                                {"qid": { "$regex": &nameToSearch, "$options": "i" }}
                                            ]
                                };
            //Search all the candidates
            
            let candidates = db.getCandidates(filter).unwrap();
            if candidates.len() > 0 {
                for candidate in candidates {
                    candidate.print_detail();
                }
            }else{
                println!("No match found :(");
            }
        }

        Some("match") => {
            let qid = matches.subcommand_matches("match").unwrap().value_of("QID").unwrap();
            match db.getPerson(&String::from(qid)) {
                Ok(_personLookingForDate) => {
                    let personLookingForDate = _personLookingForDate.unwrap();
                    println!("Matching for ...");
                    println!("{}", personLookingForDate);
                    println!("************************************************************");

                    if let Ok(_matcher) = Match::new(personLookingForDate, String::from("rules.json")) {
                        let mut matcher = _matcher.unwrap();
                        let filter = matcher.getFilter();
                        let candidates = db.getCandidates(filter).unwrap();

                        let candidatesSorted = matcher.qurate(candidates);
                        if candidatesSorted.len() > 0 {
                            for candidate in candidatesSorted {
                                println!("{}", candidate);
                            }
                        }else{
                            println!("No match found :(");
                        }
                        
                    }else{
                        println!("Policy sheet is not correct for this candidate");
                    }
                }
                
                Err(e) => {
                    println!("{}", e);
                }

            }
        }
        
        Some("insert") =>  {
            let filename = matches.subcommand_matches("insert").unwrap().value_of("INPUT").unwrap();

            // Read the data from the file
            match excel::read(filename.to_string()) {
                Ok((person_collection, warning_collection)) => {
                    //We have got the person and those field as well which doesn't feels like person
                    
                    //Print warning first
                    for warning in warning_collection {
                        println!("{}", warning);
                    }

                    
                    match db.insertAndCheckDuplicate(person_collection, true) {
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
                
                Err(e) => {
                    println!("{:?}", e);
                }    
            }
        }

        None => {
            println!("No subcommand was used");
        }

        _ => {
            println!("{}", matches.usage());
        }
    }
}
