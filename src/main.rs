use crate::result::Result;
use chrono::NaiveDateTime;
use scylla::Session;
use syslog_loose::Message;
use syslog_rfc5424::message;
//use syslog_rfc5424::SyslogMessage;
use std::net::UdpSocket;
use std::str;
use uuid::Uuid;
mod db;
mod result;
use crate::duration::Duration;
//use chrono::Duration;
mod duration;
use std::time::SystemTime;
mod log_event;


#[tokio::main]
async fn main() -> Result<()> {
    
    let s = UdpSocket::bind("127.0.0.1:10514").unwrap();
    let mut buf = [0u8; 2048];
    let session: Session = initialize().await?;
    

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
        
        
        db::add_event(&session, event).await?;
        //println!("{}", msg);

        //let msg = str::from_utf8(&buf[0..data_read]).unwrap().parse::<SyslogMessage>().unwrap();
        //println!("{:?} {:?} {:?} {:?}", msg.facility, msg.severity, msg.hostname, msg.msg);
    }
}

async fn initialize() -> Result<Session> {
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "192.168.122.206:9042".to_string());
    let session = db::create_session(&uri).await?;
    db::create_keyspace(&session).await?;
    db::create_table_log(&session).await?;
    Ok(session)
}
