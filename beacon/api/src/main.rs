use std::fs::{self, File};

fn generate_api() {
    let api_desc = lucid_beacon_api::beacon_api_mod::stub_api_description()
        .expect("Failed to get API description");

    let api = api_desc.openapi("Lucid Beacon API", lucid_beacon_api::latest_version());

    let out_dir = format!("{}/gen", env!("CARGO_MANIFEST_DIR"));
    if fs::exists(format!("{}/gen", env!("CARGO_MANIFEST_DIR"))).expect("Failed to check if gen dir exists") {
        fs::remove_dir_all(out_dir.clone()).expect("Failed to delete output directory");
    }

    fs::create_dir(out_dir.clone()).expect("Failed to create gen dir");
    let mut buffer = File::create(format!("{}/openapi.json", out_dir)).expect("Failed to create openapi.json file");

    api.write(&mut buffer).expect("Failed to write OpenAPI spec");
}

fn main() {
    generate_api();
}
