use clap::{App, Arg};
use coco::app::git_app::get_repo;
use coco::domain::config::CocoConfig;
use coco::infrastructure::name_format;
use std::fs;

fn main() {
    let matches = App::new("Coco Program")
        .version("1.0")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("Does awesome things")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches.value_of("config").unwrap_or("coco.yml");
    let contents = fs::read_to_string(config_file).expect("reading config file error");
    let config: CocoConfig = serde_yaml::from_str(&contents).expect("parse config file error");

    for x in &config.repo {
        let results = get_repo(x.url.as_str());
        let file_name = name_format::from_url(x.url.as_str());

        fs::write(file_name, results).expect("cannot write file");
    }
}
