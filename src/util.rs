use std::{fs::File, io::Read, path::Path};

use crate::route::SharedContext;

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
