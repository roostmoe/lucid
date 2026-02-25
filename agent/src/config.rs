use std::path::PathBuf;

pub fn auth_key_path(data_dir: PathBuf) -> PathBuf {
    PathBuf::from(data_dir).join("auth.key")
}

pub fn auth_cert_path(data_dir: PathBuf) -> PathBuf {
    PathBuf::from(data_dir).join("auth.crt")
}

pub fn ca_cert_path(data_dir: PathBuf) -> PathBuf {
    PathBuf::from(data_dir).join("ca.crt")
}
