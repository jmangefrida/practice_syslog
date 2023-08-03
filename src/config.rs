use std::fs;

struct Config {
    db_uri: String,
    bind_ip: String,

}

impl Config {
    fn build() -> Result<Config, &'static str> {
        read
    }
}

pub fn read_config() -> Result<Vec<String>, Box<dyn Error>> {
    let contents: String = fs::read_to_string("config/flut.conf")?;
    let lines = contents.split();
    return Ok(lines)
}