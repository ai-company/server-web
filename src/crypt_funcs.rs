use rand::RngCore;

pub const DEFAULT_COST: u32 = 10;
pub const MAX_SALT_SIZE: usize = 16;
pub const PASSWORD_HASH_SIZE: usize = 24;

pub struct HashedPassword {
    pub hash: [u8; PASSWORD_HASH_SIZE],
    pub salt: [u8; MAX_SALT_SIZE],
}

pub fn generate_from_password(password: &str) -> HashedPassword {
    let salt = {
        let mut unencoded = [0u8; MAX_SALT_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut unencoded);
        unencoded
    };
    HashedPassword {
        hash: hash_with_salt(password, &salt),
        salt,
    }
}

pub fn hash_with_salt(password: &str, salt: &[u8]) -> [u8; PASSWORD_HASH_SIZE] {
    let mut hash = [0u8; PASSWORD_HASH_SIZE];

    bcrypt::bcrypt(DEFAULT_COST, salt, password.as_bytes(), &mut hash);

    hash
}
