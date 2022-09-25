#[macro_use]
extern crate rocket;
extern crate lazy_static;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use texcreate_templates::Template;
use uuid::Uuid;

lazy_static!{
    static ref CACHE: Mutex<HashMap<String, PathBuf>> = Mutex::new(HashMap::new());
    static ref ID_: Uuid = Uuid::new_v4();
}


#[post("/", data = "<template>")]
fn make_backup(template: Json<Template>){
    let id = ID_.to_string();
    let mut cache = CACHE.lock().unwrap();
    let template = template.into_inner();
    let json = template.to_json();
    let path = PathBuf::from(&format!("backup/{}.json", &id));
    // make template.name public
    let name = template.name.clone();
    let mut file = File::create(&path).unwrap();
    file.write_all(json.as_bytes());
    let _ = cache.insert(name, path);
}

#[get("/backup")]
fn send_backup() -> Json<HashMap<String, Template>>{
    let mut templates = HashMap::new();
    let cache = CACHE.lock().unwrap();
    for (name, path) in cache.iter(){
           let template = Template::from_file(path);
           templates.insert(name.to_string(), template)
    }
    Json::from(templates)
}

#[launch]
fn rocket()  -> _{
    rocket::build().mount("/", routes![make_backup, send_backup])
}
