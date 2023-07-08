use std::net::UdpSocket;
use scylla::Session;
use crate::log_event;
use uuid::Uuid;
use crate::duration::Duration;
use crate::result::Result;
use std::str;
use std::time::SystemTime;
use crate::db;
use crate::log_parser;

pub struct SyslogListener {
    pub db_uri: String,
    pub sock_uri: String
    
}

impl SyslogListener {
    pub async fn listen(self) -> Result<()> {
        let s = UdpSocket::bind(&self.sock_uri).unwrap();
        let mut buf = [0u8; 2048];
        let session: Session = self.initialize_db().await?;
        

        loop {
            let (data_read, src_address) = s.recv_from(&mut buf).unwrap();
            let original = str::from_utf8(&buf[0..data_read]).unwrap();
            //let msg:  Message<&str> = syslog_loose::parse_message(original);
            let id = Uuid::new_v4();
            let event: log_event::LogEvent = log_event::LogEvent { 
                                                id: id,
                                                ingest_time: Duration::seconds(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64),
                                                source: src_address.to_string(),
                                                tag: "".to_string(),
                                                msg: original.to_string() };
            
            //println!("{}", event.msg);
            db::add_event(&session, &event).await?;
            log_parser::parse_syslog(event);

            //let msg = str::from_utf8(&buf[0..data_read]).unwrap().parse::<SyslogMessage>().unwrap();
            //println!("{:?} {:?} {:?} {:?}", msg.facility, msg.severity, msg.hostname, msg.msg);
        }
    }

    pub async fn initialize_db(self) -> Result<Session> {
        
        let session = db::create_session(&self.db_uri).await?;
        db::create_keyspace(&session).await?;
        db::create_table_log(&session).await?;
        Ok(session)
    }
    
}