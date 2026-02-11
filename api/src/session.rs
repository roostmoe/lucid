use rand::{RngCore, SeedableRng, rngs::StdRng};

pub(crate) fn gen_session_id() -> String {
    let mut rng = StdRng::from_os_rng();
    let mut random_bytes: [u8; 20] = [0; 20];
    rng.fill_bytes(&mut random_bytes);
    hex::encode(random_bytes)
}
