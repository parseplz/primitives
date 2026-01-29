// https://raw.githubusercontent.com/jshttp/mime-db/master/db.json

use std::collections::HashSet;
use std::fs::{
    File, {self},
};
use std::io::Write;

use serde_json::Value;

const LABELS: [&str; 8] =
    ["APP", "AUDIO", "FONT", "IMAGE", "MESSAGE", "MODEL", "TEXT", "VIDEO"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=./artifacts/db.json");
    println!("Building mime type database");
    let file_content =
        fs::read_to_string("./artifacts/db.json").expect("error reading file");
    let result: Value = serde_json::from_str::<Value>(&file_content)?;
    let mut app_vec: Vec<&str> = Vec::new();
    let mut audio_vec: Vec<&str> = Vec::new();
    let mut font_vec: Vec<&str> = Vec::new();
    let mut image_vec: Vec<&str> = Vec::new();
    let mut message_vec: Vec<&str> = Vec::new();
    let mut model_vec: Vec<&str> = Vec::new();
    let mut text_vec: Vec<&str> = Vec::new();
    let mut video_vec: Vec<&str> = Vec::new();

    if let Value::Object(map) = result {
        for entry in map.iter() {
            let extension = &entry.1["extensions"];
            if !extension.is_null()
                && let Value::Array(arr) = extension
            {
                let result: Vec<&str> = arr
                    .iter()
                    .filter_map(|x| {
                        let s = x.as_str();
                        if s.is_none() {
                            eprintln!("non-string element in array| {:?}", x);
                        }
                        s
                    })
                    .collect();
                if let Some((category, _)) = entry.0.split_once('/') {
                    match category {
                        "application" => app_vec.extend(result),
                        "audio" => audio_vec.extend(result),
                        "font" => font_vec.extend(result),
                        "image" => image_vec.extend(result),
                        "message" => message_vec.extend(result),
                        "model" => model_vec.extend(result),
                        "text" => text_vec.extend(result),
                        "video" => video_vec.extend(result),
                        _ => eprintln!("Unknown MIME category: {}", category),
                    }
                } else {
                    eprintln!(
                        "Malformed MIME type (missing '/'): {}",
                        entry.0
                    );
                }
            }
        }

        // https://github.com/jshttp/mime-db/issues/207
        // remove mp4 from application
        app_vec.retain(|&x| x != "mp4");

        // unique across vectors
        let mut vectors = vec![
            app_vec,
            audio_vec,
            font_vec,
            image_vec,
            message_vec,
            model_vec,
            text_vec,
            video_vec,
        ];

        // sort and remove duplicates
        vectors.iter_mut().for_each(|v| {
            v.sort();
            v.dedup();
        });

        vectors = unique_across_vectors(vectors);

        for (label, vec) in LABELS.iter().zip(vectors.into_iter()) {
            let string = vec_to_string(label, vec);
            if let Err(e) = write_file(label.to_lowercase().as_str(), string) {
                eprintln!("error writing file| {label}| {e}");
            }
        }
    }
    Ok(())
}

fn vec_to_string(content_type: &str, vec: Vec<&str>) -> String {
    let len = vec.len();
    let mut ct_string = format!(
        "#[rustfmt::skip]\npub const EXT_{content_type}: [&str; {len}] = ["
    );
    for ext in vec {
        ct_string.push_str(&format!(r#""{ext}","#));
    }
    ct_string.pop();
    ct_string.push_str("];");
    ct_string
}

fn write_file(name: &str, data: String) -> Result<(), std::io::Error> {
    let path = format!("./src/mime_type/{name}.rs");
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;
    file.write_all("\n".as_bytes())?;
    Ok(())
}

fn unique_across_vectors(vectors: Vec<Vec<&str>>) -> Vec<Vec<&str>> {
    let mut seen = HashSet::new();
    vectors
        .into_iter()
        .map(|vec| vec.into_iter().filter(|&item| seen.insert(item)).collect())
        .collect()
}
