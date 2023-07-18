use std::env;
use std::fs;

static CONFIG_FILE: Str = "config.conf";

fn read_filter_config() {
    let contents =fs::read_to_string(CONFIG_FILE).except("Could not read file");
}