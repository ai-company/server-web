use tokio::sync::RwLock;

use crate::{route::SharedContext, urldecode};
use chrono::prelude::*;
use std::{
    collections::HashMap,
    fs::{canonicalize, metadata, File},
    io::Read,
    path::Path,
    time::UNIX_EPOCH,
};

#[allow(dead_code)]
pub fn parse_form(body: &str) -> HashMap<String, String> {
    let mut form_data = HashMap::new();

    for line in body.split("\n") {
        let pairs: Vec<String> = line
            .trim()
            .split("=")
            .map(|i| urldecode(i.into()))
            .collect();
        form_data.insert(pairs[0].clone(), pairs[1].clone());
    }

    form_data
}

#[allow(dead_code)]
pub fn parse_query(query: &str) -> HashMap<String, String> {
    let mut form_data = HashMap::new();

    for line in query.split("&").filter(|p| !p.is_empty()) {
        let pairs: Vec<String> = line.split("=").map(|i| urldecode(i.into())).collect();
        form_data.insert(pairs[0].clone(), pairs[1].clone());
    }

    form_data
}

#[allow(dead_code)]
pub fn get_template(context: &SharedContext, path: &str) -> Option<String> {
    let path = format!("{}/template/{}.html", context.root_path, path);
    let path = Path::new(&path);
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();

    Some(String::from_utf8(buf).unwrap())
}

#[allow(dead_code)]
pub fn template_exists(context: &SharedContext, path: &str) -> bool {
    Path::new(&format!("{}/template/{}.html", context.root_path, path)).is_file()
}

#[derive(Clone)]
pub struct Asset {
    pub data: Vec<u8>,
    pub mime: String,
    pub mtime: String,
}

impl Asset {
    pub fn new(data: Vec<u8>, mime: &str, mtime: &str) -> Self {
        Self {
            data,
            mime: mime.into(),
            mtime: mtime.into(),
        }
    }
}

pub struct StaticAssetStore {
    path_root: String,
    cache: RwLock<HashMap<String, Asset>>,
}

#[allow(dead_code)]
impl StaticAssetStore {
    pub fn new(path_root: &str) -> Self {
        Self {
            path_root: path_root.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    // get real canonical path to asset, if it exists and is relative to the path root
    // pass all paths through this to prevent out-of-webroot path access
    pub fn realpath(&self, path: &str) -> Option<String> {
        if let Ok(path) = canonicalize(&format!("{}/{}", self.path_root, path)) {
            if path.to_str().unwrap().starts_with(&self.path_root) {
                Some(path.to_str().unwrap().to_owned())
            } else {
                None
            }
        } else {
            None
        }
    }

    // check if asset exists, don't lock cache for this to allow more windows for write locks
    pub fn has(&self, path: &str) -> bool {
        self.realpath(path)
            .map_or(false, |path| Path::new(&path).is_file())
    }

    // time last modified
    // returned format:
    // Tue, 15 Nov 1994 12:45:26 GMT
    pub fn mtime(&self, path: &str) -> String {
        Utc.timestamp_millis(
            metadata(&self.realpath(path).unwrap())
                .unwrap()
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64() as i64
                * 1000,
        )
        .format("%a, %e %b %Y %T GMT")
        .to_string()
    }

    // get mime type of file
    pub fn mime(&self, path: &str) -> String {
        let fspath = self.realpath(path).unwrap();
        let fspath = Path::new(&fspath);

        match fspath.extension().unwrap().to_str().unwrap() {
            // shortcuts
            "js" => "application/javascript".into(),
            "json" => "application/json".into(),
            "css" => "text/css".into(),

            // infer from content
            _ => tree_magic::from_filepath(&fspath).to_string(),
        }
    }

    // get a static web asset
    // will cache requested files in memory on request, until modified again
    pub async fn get(&self, path: &str) -> Option<Asset> {
        let mtime = self.mtime(path);

        // check cache and verify the file is unchanged
        // if it's unchanged, return the cached file, otherwise fall through and load from fs
        {
            let cache = self.cache.read().await;
            if cache.contains_key(path) {
                let asset = cache.get(path).unwrap();

                if asset.mtime == mtime {
                    // unchanged
                    return Some(asset.clone());
                } else if !Path::new(&self.realpath(path).unwrap()).is_file() {
                    // deleted
                    self.cache.write().await.remove(path);
                    return None;
                }
            }
        }

        // asset changed or not in cache
        if Path::new(&self.realpath(path).unwrap()).is_file() {
            let fspath = self.realpath(path).unwrap();
            let fspath = Path::new(&fspath);

            let mut data = Vec::new();
            File::open(fspath).unwrap().read_to_end(&mut data).unwrap();

            let mime = self.mime(path);
            let asset = Asset::new(data, &mime, &mtime);

            self.cache.write().await.insert(path.into(), asset.clone());

            Some(asset)
        } else {
            None
        }
    }
}
