use uuid::Uuid;
use crate::duration::Duration;
use scylla::FromRow;
use scylla::ValueList;
use std::collections::HashMap;

#[derive(Debug, FromRow, ValueList)]
pub struct LogEvent {
    pub id: Uuid,
    pub ingest_time: Duration,
    pub source: String,
    pub tag: String,
    pub msg: String

}

pub struct AnalyzedEvent{
    pub event: LogEvent,
    pub data: HashMap<String, String>
}