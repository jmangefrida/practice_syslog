use std::collections::HashMap;
use std::fs;
//use std::io::{BufWriter, Write, BufReader};
//use std::error::Error;
use serde_json;
//use serde_json::Value;
use serde::{Serialize, Deserialize};

use crate::parser::ParserCollection;
//use regex::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Ingester{
    pub bind_addr: String,
    pub parser: String,
    pub tags: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub db_uri: String,
    pub ingesters: Vec<Ingester>,
    pub parsers: Vec<String>

}
 impl Config {
    /*pub fn build() -> Result<Config, Box<dyn Error>> {
        print!("{:?}", std::env::current_dir());
        let config_lines: Vec<String> = read_config()?;
        Ok(Config { db_uri: config_lines[0].clone(), bind_ip: config_lines[1].clone() })
    }
    */

    pub fn build() -> Config {
        let values: String = read_config();
        let config: Config = serde_json::from_str(&values).unwrap();



        config
    }

    pub fn build_parsers(self) -> HashMap<String, ParserCollection> {
        let mut parsers: HashMap<String, ParserCollection> = HashMap::new();
        for parser in self.parsers{
            parsers.insert(parser.clone(), read_parser(&parser.clone()));
            //parsers.push(read_parser(&parser));
        }
        parsers
    }

}

pub fn read_config() -> String {
    //let file = File::open("config/flut.json").unwrap();
    //let reader = BufReader::new(file);
    //let config: Value = serde_json::from_reader(reader).unwrap();
    //println!("{}", config["db_uri"].as_str().unwrap());
    //println!("{}", config["listener"].to_string());
    let contents: String = fs::read_to_string("config/flut.json").unwrap();
    println!("File opened");
    //let config = serde_json::from_str(&contents).unwrap();
    //let lines = contents.lines().collect::<Vec<&str>>();
    //let mut config_lines:Vec<String> = vec![];

    //for line in lines {
    //    if line.chars().nth(0).unwrap() != '#' {
    //        config_lines.push(line.to_string());
    //    }
    //}

    return contents;
    
}

pub fn read_parser(filter: &str) -> ParserCollection {
    let contents: String = fs::read_to_string("config/".to_string() + filter.clone() + ".json").unwrap();
    let parser: ParserCollection = serde_json::from_str(&contents).unwrap();
    parser

}