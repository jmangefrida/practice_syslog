use scylla::{IntoTypedRows, Session, SessionBuilder};
//use uuid::Uuid;
//use syslog_loose::Message;
use crate::log_event;
use crate::result::Result;
use scylla::FromRow;
use scylla::ValueList;


static CREATE_KEYSPACE_QUERY: &str = r#"
  CREATE KEYSPACE IF NOT EXISTS logs
    WITH REPLICATION = {
      'class': 'SimpleStrategy',
      'replication_factor': 1
    };
"#;

static CREATE_LOG_TABLE_QUERY: &str = r#"
    CREATE TABLE IF NOT EXISTS logs.event (
        id UUID,
        ingest_time timestamp,
        source text,
        tag text,
        msg TEXT,
        PRIMARY KEY(id, ingest_time, source, tag)

    );
"#;

static ADD_EVENT_QUERY: &str = r#"
    INSERT INTO logs.event (id, ingest_time, source, tag, msg)
    VALUES (?, ?, ?, ?, ?);
"#;


pub async fn create_session(uri: &str) -> Result<Session> {
    SessionBuilder::new()
        .known_node(uri)
        .user("cassandra", "cassandra")
        .build()
        .await
        .map_err(From::from)
}

pub async fn create_keyspace(session: &Session) -> Result<()> {
    session
        .query(CREATE_KEYSPACE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn create_table_log(session: &Session) -> Result<()> {
    session
    .query(CREATE_LOG_TABLE_QUERY, ())
    .await
    .map(|_| ())
    .map_err(From::from)
}

pub async fn add_event(session: &Session, msg: &log_event::LogEvent) -> Result<()> {
    session
    .query(ADD_EVENT_QUERY, msg)
    .await
    .map(|_| ())
    .map_err(From::from)
}