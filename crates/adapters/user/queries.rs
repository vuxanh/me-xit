pub const SELECT_USER_PASSWORD_BY_USERNAME: &str =
    "SELECT password_hash FROM users WHERE username = $1";

pub const CREATE_USER: &str =
    "INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4)";
