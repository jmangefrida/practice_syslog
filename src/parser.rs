//Parser reads config files with regular expressions for each type of log source.
//A seperate parser is created for each definiion.  parsers are in a tree format, the message is passed into the root, partially decoded
//and then passed up the tree based on values parsed in the current level.


use std::collections::HashMap;
//use serde_json;
//use serde_json::Value;
use serde::{Serialize, Deserialize};
use serde_regex;
use regex::Regex;
use serde_json::{json, Value};

//use crate::log_event;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParserCollection {
    pub name: String,
    pub base: String,
    pub parsers: Vec<Parser>,
    pub value_type: HashMap<String, String>,

}

impl ParserCollection {
    pub fn validate_config(&self) -> Result<(), String> {
        for parser in &self.parsers {
            parser.validate_config(&self.value_type)?
        }
        
        Ok(())
    }

    pub fn parse(&self, msg: String) -> HashMap<String, Value> {
        let mut values: HashMap<String, Value> = HashMap::new();
        //let mut json_values = serde_json::from_str("{}").unwrap();
        for parser in &self.parsers {
            if parser.name == self.base {
                //print!("Running base match-");
                values.extend(parser.clone().parse(&self, &msg, &self.value_type));
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
    pub fn validate_config(&self, value_type: &HashMap<String, String>) -> Result<(), String> {
        let names = self.expression.capture_names();
        println!("{:?}", names);
        for name in names {
            match name {
                Some(n) =>{ 
                    if !value_type.contains_key(n){
                        return Err("Missing value: ".to_string() + n)
                    }
                },
                None => ()
            };



            
        }

        Ok(())
    }

    pub fn parse(self, parser_coll:&ParserCollection, msg: &String, value_type: &HashMap<String, String>) -> HashMap<String,Value> {
        //print!("Running match {}", self.name);
        let mut curser: usize = 0;
        let mut values: HashMap<String, Value> = HashMap::new();
        let mut names = self.expression.capture_names();
        let Some(caps) = self.expression.captures(&msg) else {
            //return log_event::AnalyzedEvent{ event: event, data: values, log_type: log_event::LogType::UNKNOWN}
            return values
        };

        for cap in caps.iter() {
            //println!("{:?}-{:?}",names.next().unwrap(), cap.unwrap_or(""));
            match names.next().unwrap() {
                Some(na) => {
                    curser = cap.unwrap().end();
                    let v = match value_type[na].as_str() {
                        "int" => json!(cap.unwrap().as_str().to_string().parse::<i64>().unwrap()),
                        "float" => json!(cap.unwrap().as_str().to_string().parse::<f64>().unwrap()),
                        "string" | &_ => json!(cap.unwrap().as_str().to_string()),
                    };
                    values.insert(na.to_string(), v)
                },
                None => Some(json!(String::new()))
            };

        }
        if values.contains_key(&self.decision) {
            for branch in self.branches {
                if branch.value == values[&self.decision] {
                    for parser in &parser_coll.parsers {
                        if parser.name == branch.name {

                            values.extend(parser.clone().parse(parser_coll, &msg[curser..].to_string(), value_type));
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
    //value_type: String,
    name: String
}

//pub enum ValueType {
//    String,
//    Int,
//    Float
//}
