pub mod excel;
pub mod db;
pub mod qdmatch;


use clap::{Arg, App, SubCommand};
use db::{db_gateway};
use qdmatch::matcher::Match;

fn main() {

    //Create Database
    let mut db = db_gateway::DbGateway::new("mongodb://localhost:27017");

    //Connect Database
    if let Err(e) = db.connect() {
        println!("{:?}", e);
        return;
    }

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
        .subcommand(SubCommand::with_name("match")
            .about("Match person")
            .version("0.0")
            .arg(Arg::with_name("QID")
                .help("QID of the person for whom the date is to be qurated.")
                .required(true)
                .index(1)))
        .get_matches();


        
    

    //Check if Excel update is requested
    match matches.subcommand_name() {

        Some("match") => {
            let qid = matches.subcommand_matches("match").unwrap().value_of("QID").unwrap();
            let personLookingForDate = db.getPerson(&String::from(qid)).unwrap().unwrap();
            println!("Matching for ...");
            println!("{:<10} {:<40} {:<5} {:<10}", personLookingForDate.qid, personLookingForDate.name, personLookingForDate.gender, personLookingForDate.age);
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
        
        Some("update") =>  {
            let filename = matches.subcommand_matches("update").unwrap().value_of("INPUT").unwrap();

            // Read the data from the file
            match excel::read(filename.to_string()) {
                Ok((person_collection, warning_collection)) => {
                    //We have got the person and those field as well which doesn't feels like person
                    
                    //Print warning first
                    for warning in warning_collection {
                        println!("{}", warning);
                    }

                    
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
