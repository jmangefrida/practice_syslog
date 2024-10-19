use crate::log_event::{DsEvent, self};
use crate::query::Filter;
use serde::{Deserialize, Serialize};
use serde_json::value::Index;
use serde_json::{json, Value, map};
//use core::slice::SlicePattern;
//use tokio::fs;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, BufReader, Write, prelude::*, SeekFrom};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};


static EXCLUSIONS: &'static [&str] = &["DATETIME"];

pub struct DSWriter {
    pub name: String,
    pub path: String,
    pub file_name: String,
    //pub receiver: mpsc::Receiver<serde_json::Value>,
    pub index: HashMap<String, HashMap<String, Vec<usize>>>,
    //pub index: HashMap<String, serde_json::map::Map>
}

impl DSWriter {
    pub fn new(
        name: String,
        path: String,
        //receiver: mpsc::Receiver<serde_json::Value>,
    ) -> DSWriter {
        DSWriter {
            name: name.clone(),
            path: path + &"/".to_owned() + &name,
            file_name: "".to_string(),
            //receiver: receiver,
            index: HashMap::new(),
        }
    }

    pub fn start(&mut self, receiver: mpsc::Receiver<log_event::DsEvent>) {
                
        //self.fix_last_file();
        //self.index = HashMap::new();
        //let index: HashMap<String, HashMap<String, usize>> = HashMap::new();
        //let icounter: usize;
        //let v_index: Vec<Vec<usize>>;
        let mut file_time = Self::gen_inst_time() / 300 * 300;
        let mut new_file_time: i64;
        let mut file_name: String = Self::gen_file_name();
        if std::path::Path::new(&(self.path.clone() + "/" + &file_name)).exists() {
            self.index_file(self.path.clone() + "/" + &file_name, false);
        }
        let mut file: File = self.open_file(self.path.clone(), Self::gen_file_name());
        
        let mut writer: std::io::BufWriter<File> = std::io::BufWriter::new(file);
        let marker = "\u{0003}";
        let mut counter: usize = 0;
        

        let events = receiver.iter();

        for event in events {
            new_file_time  = Self::gen_inst_time() / 300 * 300;
            
            if new_file_time != file_time {
                self.save_index(self.path.clone(), file_name.clone());
                file_time = new_file_time;
                file_name = Self::gen_file_name();
                drop(writer);
                

                file = self.open_file(self.path.clone(), file_name.clone());
                self.file_name = file_name.clone();
                writer = std::io::BufWriter::new(file);
            }
            //counter += 1;
            self.add_to_index(&event, counter);

            let buff = serde_json::to_string(&event).unwrap() + marker;

            counter += buff.len();
            writer
                .write_all(&buff.as_bytes())
                .unwrap();

            
            drop(event);

        }
        //Add to index

        
    }

    fn add_to_index(&mut self, data: &log_event::DsEvent, counter: usize) {
        
        let msg = &data.msg;
        //if msg.is_object() {
        //    println!("true");
        //}
        for (field, value) in msg.as_object().unwrap() {
        
            if EXCLUSIONS.contains(&&field.as_str()) {
                continue;
            }
            let svalue: String = value.to_string();
            if let Some(exist_field) = self.index.get_mut(field) {
                if let Some(exist_value) = exist_field.get_mut(&svalue) {
                    //*exist_value += 1;
                    exist_value.push(counter);
                    continue;
                }

                exist_field.insert(svalue, vec![counter]);
                continue;
            }
            let mut ifield: HashMap<String, Vec<usize>> = HashMap::new();
            ifield.insert(svalue, vec![counter]);
            self.index.insert(field.to_string(), ifield);
            
        }
        
        let msg = ();

    }

    fn save_index(&mut self, path: String, file_name: String) {

        let mut counter: usize = 0;
        let mut ixfile = File::create(path.clone() + "/" + &file_name + "x").unwrap();
        let mut writer: std::io::BufWriter<File> = std::io::BufWriter::new(ixfile);
        
        

        for (field_k, field_v) in &mut self.index {
            for (value_k, value_v) in field_v {
                
                let buf: Vec<u8> = bincode::serialize(&value_v).unwrap();
                let length = buf.len();
                writer.write_all(&buf);
                value_v.clear();
                value_v.append(&mut vec![counter, length]);
                counter += length;
                
            }
        }
        drop(writer);
        //drop(ixfile);
        println!("{file_name}");
        let mut ifile = File::create(path + "/" + &file_name + "i").unwrap();
        //let ibuff = json!(self.index);
        let ibuff = bincode::serialize(&self.index).unwrap();
        //ifile.write_all(ibuff.to_string().as_bytes()).unwrap();
        ifile.write_all(&ibuff).unwrap();
        drop(ibuff);
        drop(ifile);
        self.index.clear();
        self.index.shrink_to(1);
    }

    fn open_file(&self, path: String, name: String) -> File {
        let full_name = path + "/" + &name;
        let pos = full_name.rfind("/").unwrap();
        std::fs::create_dir_all(&full_name[0..pos]).unwrap();
        let file: File;
        if std::fs::metadata(full_name.clone()).is_ok() {
            file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(full_name.clone())
                .unwrap();
        } else {
            file = File::create(full_name.clone()).unwrap();
        }
        //self.file_name = name.clone();
        file
    }

    pub fn gen_file_name() -> String {
        let inst_time =  DSWriter::gen_inst_time();
        let file_time = inst_time / 300 * 300;
        let folder_time = inst_time / 86400 * 86400;
        folder_time.to_string() + "/" + &file_time.to_string() + ".ds"

    }

    pub fn gen_inst_time() -> i64 {
        SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        
    }

    //pub fn fix_last_file(&mut self) -> bool {
    //    let mut files: Vec<String>;
    //    let mut paths: Vec<String> = std::fs::read_dir(&self.path).unwrap().map(|f| f.unwrap().path().display().to_string()).collect();
    //    paths.sort();
    //    
    //    let last_path = paths.last();
    //    match last_path {
    //        Some(n) => files = std::fs::read_dir(&n).unwrap().map(|f| f.unwrap().file_name().into_string().unwrap()).collect(),
    //        None => return false
    //    }
//
    //    files.sort();
//
    //    let last_file = files.last();
    //    let bresult = match last_file {
    //        Some(n) => self.index_file(n.clone()),
    //        None => false
    //    };
//
    //    println!("path:{:?}", paths.last());
    //    true
    //}

    pub fn index_file(&mut self, file_name: String, save: bool) -> bool {
        let contents = match std::fs::read_to_string(&file_name.clone()) {
            Ok(content) => content,
            Err(_) => "fail".to_string()
        };

        let events = contents.split("\u{0003}");
        let counter: usize = 0;
        for event in events {
            if event.len() == 0 {
                break;
            }
            let s_event: DsEvent = serde_json::from_str(&event).unwrap();
            self.add_to_index(&s_event, counter);
            
        }

        if save {
            self.save_index(self.path.clone(), file_name.clone());
        }

        true
    }

  
}

pub struct DSReader {
    pub name: String,
    pub path: String,
    pub time: i64
}


impl DSReader {
    pub fn new(name: String, path: String, time: i64) -> DSReader {

        DSReader  {
            name: name.clone(),
            path: path + &"/".to_owned() + &name,
            time: time,
        }
    }

    pub fn check_file_exists(file_name: String) -> u8 {
        let mut count: u8 = 0;
        if std::path::Path::new(&(file_name.clone())).exists() {
            count = 1;
            if std::path::Path::new(&(file_name.clone() + "i")).exists() {
                count = 2;
                if std::path::Path::new(&(file_name + "x")).exists() {
                    count = 3;
                }
            }
        }

        count
        
    }

    pub fn search(&self, values: &mut HashMap<String, Filter>)  -> Result<HashMap<String, Vec<(i64, usize)>>, ()> {
        let folder_time: i64 = self.time / 86400 * 86400;
        let file_name = self.path.clone() + "/" + &folder_time.to_string() + "/" + &self.time.to_string() + ".ds";
        let exists: u8 = Self::check_file_exists(file_name.clone());
        let mut event_map: HashMap<String, Vec<(i64, usize)>> = HashMap::new();
        if exists == 3{
            let contents = match std::fs::read(&(file_name.clone() + "i") ) {
                Ok(content) => content,
                Err(_) => panic!("failed to read index file.")
            };

            let index: HashMap<String, HashMap<String, Vec<usize>>> = bincode::deserialize(&contents).unwrap();
            for (K, V) in values{
                if !index.contains_key(&V.field) {
                    continue;
                }
                if !index[&V.field].contains_key(&V.value.to_string()) {
                    continue;
                }
                let bin_location = index[&V.field][&V.value.to_string()].clone();
                let mut bin_file = OpenOptions::new()
                                    .read(true)
                                    .open(file_name.clone() + "x")
                                    .unwrap();
                //bin_file.seek(SeekFrom::Current(bin_location[0] as i64)).unwrap();
                
                bin_file.seek(SeekFrom::Start(bin_location[0] as u64)).unwrap();
                //bin_file.
                println!("pos:{}", bin_file.stream_position().unwrap());
                println!("pointers:{:?}", bin_location);
                let mut ibuf =  vec![0u8; bin_location[1]];
                println!("buf size:{:?}", ibuf.len());
                bin_file.read_exact(&mut ibuf).unwrap();
                println!("buf contents:{:?}", ibuf);
                //let mut ibuf:Vec<u8> = vec![];
                //bin_file.read_to_end(&mut ibuf).unwrap();
                //let bin_reader: BufReader::new<bin_file>;
                let event_index: Vec<usize> = bincode::deserialize(&ibuf).unwrap();
                drop(ibuf);
                let mut event_list: Vec<(i64, usize)> = Vec::new();
                
                for event in event_index{
                    event_list.push((self.time, event));
                    //let value: &Filter = values[K];
                    //let value = values[K].hash_result.as_ref().unwrap().push((self.time, event));
                    //values[K].hash_result.unwrap().push((self.time, event));
                }
                event_map.insert(K.to_string(), event_list);
            }

            

            return Ok(event_map);
        }

        return(Err(()));

    }

//    pub fn search(&self, values: HashMap<String, String>) -> Result<HashMap<String, Vec<(i64, usize)>>, ()> {
//        let folder_time: i64 = self.time / 86400 * 86400;
//        let file_name = self.path.clone() + "/" + &folder_time.to_string() + "/" + &self.time.to_string() + ".ds";
//        let exists: u8 = Self::check_file_exists(file_name.clone());
//        let mut event_map: HashMap<String, Vec<(i64, usize)>> = HashMap::new();
//        if exists == 3{
//            let contents = match std::fs::read(&(file_name.clone() + "i") ) {
//                Ok(content) => content,
//                Err(_) => panic!("failed to read index file.")
//            };
//
//            let index: HashMap<String, HashMap<String, Vec<usize>>> = bincode::deserialize(&contents).unwrap();
//            for (K, V) in values{
//                if !index.contains_key(&K) {
//                    continue;
//                }
//                if !index[&K].contains_key(&V) {
//                    continue;
//                }
//                let bin_location = index[&K][&V].clone();
//                let mut bin_file = OpenOptions::new()
//                                    .read(true)
//                                    .open(file_name.clone() + "x")
//                                    .unwrap();
//                //bin_file.seek(SeekFrom::Current(bin_location[0] as i64)).unwrap();
//                
//                bin_file.seek(SeekFrom::Start(bin_location[0] as u64)).unwrap();
//                //bin_file.
//                println!("pos:{}", bin_file.stream_position().unwrap());
//                println!("pointers:{:?}", bin_location);
//                let mut ibuf =  vec![0u8; bin_location[1]];
//                println!("buf size:{:?}", ibuf.len());
//                bin_file.read_exact(&mut ibuf).unwrap();
//                println!("buf contents:{:?}", ibuf);
//                //let mut ibuf:Vec<u8> = vec![];
//                //bin_file.read_to_end(&mut ibuf).unwrap();
//                //let bin_reader: BufReader::new<bin_file>;
//                let event_index: Vec<usize> = bincode::deserialize(&ibuf).unwrap();
//                drop(ibuf);
//                let mut event_list: Vec<(i64, usize)> = Vec::new();
//                
//                for event in event_index{
//                    event_list.push((self.time, event));
//                }
//                event_map.insert(K, event_list);
//            }
//
//            
//
//            return Ok(event_map);
//        }
//
//        return(Err(()));
//
//    }
}
