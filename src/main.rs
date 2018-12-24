#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rocket_contrib;
extern crate glob;
extern crate open;

use glob::glob;
use rocket_contrib::templates::{Template};
use rocket::config::{Config, Environment};
use std::env;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Serialize)]
struct Document {
    path: String,
    contents: String
}

#[derive(Serialize)]
struct TemplateContext {
    files: String
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &TemplateContext {
        files: serde_json::to_string(&get_docs()).unwrap()
    })
}

fn launchpad() -> rocket::Rocket {
    let config = Config::build(Environment::Production)
        .address("localhost")
        .port(8459)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .mount("/", routes![index])
        .attach(Template::fairing())
}

fn main() {
    if !open::that("http://localhost:8459").is_ok() {
        println!("Error opening browser");
    }
    launchpad().launch();
}

fn get_folder_path() -> io::Result<String> {
    let app_dir = env::current_dir();
    let app_dir = match app_dir {
        Result::Ok(s) => format!("{}", s.as_os_str().to_str().unwrap()),
        Result::Err(_) => format!("{}", "./*.md"),
    };

    let folder_config = format!("{}\\folder.txt", app_dir);

    if Path::new(&folder_config).exists() {
        println!("Path {} exists, reading", folder_config);
        let result = fs::read_to_string(folder_config);
        return match result {
            Result::Err(_) => Ok(format!("{}", "./*.md")),
            Result::Ok(s) => Ok(format!("{}", s)),
        };
    }

    return Ok(format!("{}", "./*.md"));
}

fn get_docs() -> Vec<Document> {
    let mut docs: Vec<Document> = vec![];
    let root = get_folder_path().unwrap();

    println!("Reading files at {}", root);

    for path in glob(&root).expect("Failed to read glob pattern for files") {
        match path {
            Ok(path) => docs.push(Document {
                path: path.as_path().as_os_str().to_str().unwrap().to_string(),
                contents: fs::read_to_string(path).unwrap()
            }),
            Err(e) => println!("Error reading path: {:?}", e)
        }
    }

    return docs
}