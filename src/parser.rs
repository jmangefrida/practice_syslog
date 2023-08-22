//Parser reads config files with regular expressions for each type of log source.
//A seperate parser is created for each definiion.  parsers are in a tree format, the message is passed into the root, partially decoded
//and then passed up the tree based on values parsed in the current level.


use std::collections::HashMap;
//use serde_json;
//use serde_json::Value;
use serde::{Serialize, Deserialize};
use serde_regex;
use regex::Regex;


//use crate::log_event;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParserCollection {
    pub name: String,
    pub base: String,
    pub parsers: Vec<Parser>

}

impl ParserCollection {
    pub fn parse(self, msg: String) -> HashMap<String, String> {
        let mut values = HashMap::new();
        for parser in &self.parsers {
            if parser.name == self.base {
                //print!("Running base match-");
                values.extend(parser.clone().parse(&self, &msg));
                break;
            }
        }
        
        //return log_event::AnalyzedEvent { event: event, data: values , log_type: self.name }
        return values
   }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parser {
    name: String,
    #[serde(with = "serde_regex")]
    expression: Regex,
    decision: String,
    branches: Vec<Branch>
}

impl Parser {
    pub fn parse(self, parser_coll:&ParserCollection, msg: &String) -> HashMap<String,String> {
        //print!("Running match {}", self.name);
        let mut values = HashMap::new();
        let mut names = self.expression.capture_names();
        let Some(caps) = self.expression.captures(&msg) else {
            //return log_event::AnalyzedEvent{ event: event, data: values, log_type: log_event::LogType::UNKNOWN}
            return values
        };

        for cap in caps.iter() {
            //println!("{:?}-{:?}",names.next().unwrap(), cap.unwrap_or(""));
            match names.next().unwrap() {
                Some(na) => {values.insert(na.to_string(), cap.unwrap().as_str().to_string())},
                None => Some(String::new())
            };

        }
        if values.contains_key(&self.decision) {
            for branch in self.branches {
                if branch.value == values[&self.decision] {
                    for parser in &parser_coll.parsers {
                        if parser.name == branch.name {

                            values.extend(parser.clone().parse(parser_coll, msg));
                            return values
                        }
                    }
                }
            }
        }

        return values
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Branch {
    value: String,
    value_type: String,
    name: String
}

//pub enum ValueType {
//    String,
//    Int,
//    Float
//}
