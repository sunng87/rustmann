extern crate protoc_rust;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use protoc_rust::Customize;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    protoc_rust::run(protoc_rust::Args {
        out_dir: &out_dir,
        input: &["protos/riemann.proto"],
        includes: &["protos"],
        customize: Customize {
            carllerche_bytes_for_bytes: Some(true),
            carllerche_bytes_for_string: Some(true),
            ..Default::default()
        },
    })
        .expect("protoc");

    // Create mod.rs accordingly
    let mod_file_content = ["protos/riemann.proto"]
        .iter()
        .map(|proto_file| {
            let proto_path = Path::new(proto_file);
            format!(
                "pub mod {};",
                proto_path
                    .file_stem()
                    .expect("Unable to extract stem")
                    .to_str()
                    .expect("Unable to extract filename")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut mod_file = File::create(Path::new(&out_dir).join("mod.rs")).unwrap();
    mod_file
        .write_all(mod_file_content.as_bytes())
        .expect("Unable to write mod file");
}
