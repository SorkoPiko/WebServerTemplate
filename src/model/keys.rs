use hkdf::Hkdf;
use sha2::Sha256;

pub struct Keys {
    pub jwt_secret: String,
}

impl Keys {
    pub fn from_master_key(master_key: &str) -> Self {
        let hk = Hkdf::<Sha256>::new(None, master_key.as_bytes());

        let mut jwt_secret = [0u8; 32];
        hk.expand(b"jwt", &mut jwt_secret).unwrap();

        Self {
            jwt_secret: hex::encode(jwt_secret),
        }
    }
}