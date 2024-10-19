use std::hash::Hash;
use std::ops::IndexMut;
use std::{collections::HashMap, sync::mpsc};
use std::{thread, time};

use serde_json::{json, Value};
use syn::parse_quote_spanned;
use crate::log_event::LogEvent;

use serde::{Serialize, Deserialize};
use crate::datastore;
const EQUALITY: [&'static str; 5] = ["=", "<", ">", "<=", ">="];
const CONJUNC: [&'static str; 4] = ["AND", "OR", "(", ")"];


//pub struct DSQuery {
//    pub query: String,
//    pub query_parts: Vec<String>,
//    pub filters: HashMap<String, DSFilter>
//}

//pub struct DSFilter {
//    pub id: String,
//    pub field: String,
//    pub equality: String,
//    pub value: Value,
//    pub result: Vec<(i64, usize)>
//}

pub struct Query {
    pub query: String,
    pub query_parts: Vec<String>,
    pub filters: HashMap<String, Filter>,
    //pub value_type: HashMap<String, String>



}
#[derive(Clone)]
pub struct Filter {
    pub id: String,
    pub field: String,
    pub equality: String,
    pub value: Value,
    pub result: bool,
    pub vec_result: Vec<(i64, usize)>
    //pub hash_result: Option<Vec<(i64, usize)>>
}

impl Filter {
    pub fn update_vec_result(&mut self, list: Vec<(i64, usize)>) {
        self.vec_result.append(&mut list.clone());
    }
}

//impl IndexMut<Idx> for Filter {
//    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
//        match index {
//            Idx = 
//        }
//    }
//}

//enum Equality {
//    Equal = "=",
//    Lt = "<",
//    Gt = ">",
//    LtE = "<=",
//    Gte = ">="
//}

//enum Conjunc {
//    And = "and",
//    Or = "or"
//}
//

impl Query {
    pub fn new(query: String) -> Query {
        let (query_parts, filters) = Query::parse(query.clone());
        Query { query: query, query_parts: query_parts, filters: filters}
    } 
    pub fn parse(mut query: String) -> (Vec<String>, HashMap<String, Filter>) {
        //self.query = self.query.replace("= ", "=").replace(" =", "=");
        for eq in EQUALITY {
            query = query.replace(&format!(" {eq}"), eq).replace(&format!("{eq} "), eq);
        }
        query = query.replace(" and ", " AND ").replace(" or ", " OR ");
        query = query.replace("(", " ( ").replace(")", " ) ");

        //Break the query string into its components so "user=bob or user=susan" becomes ["user=bob", "OR", "user=susan"]
        let mut parts: Vec<String> = query.split_ascii_whitespace().map(|x| x.to_string()).collect();
        parts = Query::reorder(parts);

        //let mut filter_query: Vec<String> = vec![];
        //let mut filters:  HashMap<String, Filter> = HashMap::new();
        
        
        Query::create_filters(parts)
        




            
    }

    fn create_filters(mut parts: Vec<String>) -> (Vec<String>, HashMap<String, Filter>) {
        let mut filters: HashMap<String, Filter> = HashMap::new();
        for i in 0.. parts.len() {
            for eq in EQUALITY {
                if parts[i].contains(eq) {
                    let filter_parts: Vec<&str> = parts[i].split(eq).collect();
                    println!("filter parts:{:?}", filter_parts);
                    
                    let parsed_value: Value =  match filter_parts[1].parse::<f64>() {
                        Ok(v) => json!(v),
                        Err(_v) => json!(filter_parts[1]),
                    };
                    
                    let filter: Filter = Filter { 
                        id: uuid::Uuid::new_v4().to_string(),
                        field: filter_parts[0].to_owned(), 
                        equality: eq.to_string(), 
                        value: parsed_value, 
                        result: false,
                    vec_result: vec![] };
                    parts[i] = filter.id.clone();
                    println!("Created filter:{}:{}:{}", filter.field, filter.equality, filter.value);

                    filters.insert(filter.id.clone(), filter);
                }
            }
        }

        (parts, filters)



            
        }
    

    fn reorder(steps: Vec<String>) -> Vec<String> {
        let mut temp: Vec<String> = vec![];
        let mut new_steps: Vec<String> = vec![];
        for i in steps.iter() {
            //println!("{:?}, {:?}", temp, i);
            match i.as_str() {
                "(" => {temp.push(i.to_string()); },
                ")" => {
                    while temp.last().unwrap() != &"(" {
                        new_steps.push(temp.pop().unwrap());
                    }
                    temp.pop().unwrap();
                }
                "AND"|"OR" => {
                    if temp.is_empty() {
                        temp.push(i.to_string());
                        continue;
                    }
                    if Self::priority(&i.to_string()) > Self::priority(temp.last().unwrap()){
                        temp.push(i.to_string());
                    }else{
                        while Self::priority(&i.to_string()) <= Self::priority(temp.last().unwrap()){
                            new_steps.push(temp.pop().unwrap());
                            if temp.last() == None {
                                break;
                            }
                        }
                        temp.push(i.to_string());
    
                    }
                },
                _ => {new_steps.push(i.to_string());}
    
            }
    
            
        }
        while !temp.is_empty() {
            new_steps.push(temp.pop().unwrap());
        }
    
        return new_steps;
    }

    fn priority(op: &str) -> i32{
        match op {
            "OR" => 1,
            "AND" => 2,
            _ => 0
        }
    }

    fn calculate(eq: Vec<String>) -> bool{
        let mut result: Vec<String> = vec![];
        for i in eq{
            match i.as_str() {
                "AND"|"OR" => { 
                    let oper = i;
                    let op2 = result.pop().unwrap();
                    let op1 = result.pop().unwrap();
                    let val = Query::operation(oper, op1, op2);
                    result.push(val.to_string());
    
                }
                _ => {result.push(i.to_string())}
            }
        }
    
        result.pop().unwrap().parse::<bool>().unwrap()
    }

    fn operation(oper: String, op1: String, op2:String) -> bool {
        let op1 = op1.parse::<bool>().unwrap();
        let op2 = op2.parse::<bool>().unwrap();
        match oper.as_str() {
            "AND" => op1 && op2,
            "OR" => op1 || op2,
            _ => false
            
        }
    }

    pub fn check(&self, event: HashMap<String, Value>) -> bool {
        //Make a copy of the query string because we don't want to change it for the instance
        let mut query_parts = self.query_parts.clone();

        //Go through each piece of the query and replace the filter references with boolean results
        for i in 0.. query_parts.len() {
            let query_part = &query_parts[i].clone();
            if !CONJUNC.contains(&query_part.as_str()) {
                let filter = &self.filters[query_part];
                
                if event.contains_key(&filter.field) {
                    query_parts[i] = Query::check_operation(filter, event[&filter.field].clone()).to_string();
                } else {
                    query_parts[i] = "false".to_string();
                }
            }
        }

        Query::calculate(query_parts)

        
    }

    //fn update_vec_result(&mut self, id:String, list: Vec<(i64, usize)>) {
    //    let mut filter = self.filters[&id].clone();
    //    filter.update_vec_result(list);
    //    self.filters[&id] = filter;
    //    //self.filters[&id].update_vec_result(list.clone());
    //}

    fn check_operation(filter: &Filter, event: Value) -> bool {
        match filter.equality.as_str() {
            "=" => if event.is_number() {filter.value.as_f64() == event.as_f64()} else {filter.value.as_str() == event.as_str()},
            "<" => if event.is_number() {filter.value.as_f64() < event.as_f64()} else { false},
            ">" => if event.is_number() {filter.value.as_f64() > event.as_f64()} else { false},
            "<=" => if event.is_number() {filter.value.as_f64() <= event.as_f64()} else { false},
            ">=" => if event.is_number() {filter.value.as_f64() >= event.as_f64()} else { false},
            &_ => false,
        }
    }

    //fn vec_calculate()
    
}

pub struct SearchQuerier{
    query: Query,
    datastore: String,
    path: String,
    start_time: i64,
    end_time: i64,
    
}

impl SearchQuerier {
    pub fn new(query: String, datastore: String, path: String, start_time: i64, end_time: i64) -> SearchQuerier {
        SearchQuerier { query: Query::new(query), datastore: datastore, path: path, start_time: start_time, end_time: end_time }
    }

    pub fn search(&mut self) -> Result<Vec<(i64, usize)>, ()> {
        let mut time_files: Vec<i64> = vec![];
        let mut time_check: i64 = self.start_time / 300 * 300;
        while time_check < self.end_time {
            time_files.push(time_check.clone());
            time_check = time_check + 300;
        }

        //let ds_list: Vec<datastore::DSReader> = vec![];
        let mut ds_event_map: HashMap<String, Vec<(i64, usize)>> = HashMap::new();

        for time in time_files {
            let ds = datastore::DSReader::new(self.datastore.clone(), self.path.clone(), time);
            let event_map = ds.search(&mut self.query.filters).unwrap();
            for (field, list) in event_map {
                if ds_event_map.contains_key(&field) {
                    let mut tmp: Vec<(i64, usize)> = ds_event_map[&field].clone();
                    tmp.append(&mut list.clone());
                    ds_event_map.remove(&field);
                    ds_event_map.insert(field, list);
                    
                } else {
                    ds_event_map.insert(field, list);
                }
            }
            

        }

        Ok((Vec::new()))

    }
}

pub struct RealtimeQuerier{
    queries: Vec<(Query,ACTION)>,
    receiver: mpsc::Receiver<HashMap<String, Value>>
}

impl RealtimeQuerier {
    pub fn new(queries:Vec<(String, ACTION)>, receiver: mpsc::Receiver<HashMap<String, Value>>) -> RealtimeQuerier {
        
        let parsed_queries: Vec<(Query, ACTION)> = queries.into_iter().map(|x| (Query::new(x.0), x.1)).collect();
        
        RealtimeQuerier { queries: parsed_queries, receiver: receiver}
    }

    pub fn start(&self) {
        loop {
            //println!("running!");
            //let dur = time::Duration::from_millis(10);
            //thread::sleep(dur);
            let msg = self.receiver.recv().unwrap();
            for (q, a) in &self.queries {
                if q.check(msg.clone()) {
                    //println!("{:?}", a);
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ACTION {
    ALERT(String),
    LOG(String),
}