#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::{create_dir, File, read_to_string, remove_dir_all};
use std::io::Write;
use std::path::PathBuf;
use tokio::sync::Mutex;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use texcreate_templates::Template;
use uuid::Uuid;
use zip::ZipWriter;
use lazy_static::lazy_static;

lazy_static!{
    static ref CACHE: Mutex<HashMap<String, PathBuf>> = Mutex::new(HashMap::new());
}


#[post("/new", data = "<template>")]
async fn make_backup(template: Json<Template>){
    let id = Uuid::new_v4().to_string();
    let mut cache = CACHE.lock().await;
    let template = template.into_inner();
    let json = template.to_json();
    if !PathBuf::from("backup").exists(){
        create_dir("backup").unwrap()
    }
    let path = PathBuf::from(&format!("backup/{}.json", &id));
    // make template.name public
    let name = template.name();
    let mut file = File::create(&path).unwrap();
    file.write_all(json.as_bytes()).unwrap();
    let _ = cache.insert(name, path);
}

#[get("/backup")]
async fn send_backup() -> Json<HashMap<String, Template>>{
    let mut templates = HashMap::new();
    let cache = CACHE.lock().await;
    for (name, path) in cache.iter(){
           let template = Template::from_file(path.clone());
           templates.insert(name.to_string(), template);
    }
    Json::from(templates)
}
#[get("/download/<name>")]
async fn download_backup(name: &str) -> Option<NamedFile>{
    let path = PathBuf::from("download_backup");
    if path.exists(){
        // clean directory and recreate
        remove_dir_all(&path).unwrap();
        create_dir(&path).unwrap();
    } else{
        create_dir(&path).unwrap()
    }

    let file_path = path.join(&format!("{}.zip", name));
    let mut zip = ZipWriter::new(File::create(&file_path).unwrap());
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let cache = CACHE.lock().await;

    match name{
        "all" => {
            for path in cache.values(){
                zip.start_file(path.to_str().unwrap(), options).unwrap();
                let contents = read_to_string(path).unwrap();
                zip.write_all(contents.as_bytes()).unwrap();
            }
        }
        _ => {
            let json_path = match cache.get(name){
                Some(p) => p,
                None => return None
            };
            zip.start_file(format!("{}.json", name), options).unwrap();
            let content = read_to_string(json_path).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
    }
    let _ = zip.finish().unwrap();
    NamedFile::open(&file_path).await.ok()
}


#[get("/list")]
async fn get_list() -> String{
    let cache = CACHE.lock().await;
    let mut vec = Vec::new();
    for (name, _) in cache.iter(){
        let s = format!(" - {name}");
        vec.push(s)
    }
    vec.join("\n")
}

#[get("/")]
fn index() -> String{
    "All texcreate templates are stored here!!!".to_string()
}

#[launch]
fn rocket()  -> _{
    rocket::build().mount("/", routes![make_backup, send_backup, download_backup, index, get_list])
}
