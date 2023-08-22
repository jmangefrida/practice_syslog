//use std::collections::HashMap;
use std::net::UdpSocket;
use scylla::Session;
use crate::log_event;
use uuid::Uuid;
use crate::duration::Duration;
use crate::result::Result;
use std::str;
use std::time::SystemTime;
use crate::db;
use crate::parser;
//use crate::filter;
use std::time;

#[derive(Clone)]
pub struct SyslogListener {
    pub db_uri: String,
    pub sock_uri: String,
    pub parser: parser::ParserCollection,
    pub tags: Vec<String>,
    
}

impl SyslogListener {
    pub async fn listen(self) -> Result<()> {
        println!("connecting to IP:{}", self.sock_uri);
        let s = UdpSocket::bind(&self.sock_uri).unwrap();
        let mut buf = [0u8; 2048];
        let session: Session = self.initialize_db().await.expect("Error connecting to database.");

        
        loop {
            //print!("looping");
            let (data_read, src_address) = s.recv_from(&mut buf).unwrap();
            let original = str::from_utf8(&buf[0..data_read]).unwrap();
            //let msg:  Message<&str> = syslog_loose::parse_message(original);
            let id = Uuid::new_v4();
            let start = time::Instant::now();
            let event: log_event::LogEvent = log_event::LogEvent { 
                                                id: id,
                                                ingest_time: Duration::seconds(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64),
                                                source: src_address.to_string(),
                                                tag: "".to_string(),
                                                msg: original.to_string(),
                                                data: self.parser.clone().parse(original.to_string()),
                                                log_type: self.parser.name.clone() };
            
            //println!("{}", event.msg);
            //let mut analyzed_event = self.parser.clone().parse(event);
            //db::add_event(&session, &analyzed_event).await?;
            db::add_event(&session, &event).await.expect("DB Error");
            let tduration = start.elapsed();
            println!("timing:{:?}", tduration);
            println!("{:?}", event.data);

        }
    }

    pub async fn initialize_db(&self) -> Result<Session> {
        
        let session = db::create_session(&self.db_uri).await?;
        db::create_keyspace(&session).await?;
        db::create_table_log(&session).await?;
        //db::update_table_log(&session).await?;
        Ok(session)
    }
    
}