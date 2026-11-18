pub const CREATE_USER: &str = r"
INSERT INTO users (id, username, password_hash, created_at)
VALUES
('e16a6dfe-4696-4169-9bdb-a52b1d4acb15', 'hello', '$argon2id$v=19$m=19456,t=2,p=1$mjiCrWv/w4JQfpza3S3JCw$+5ZaldBNSMj67WzDstF2xISFUOG4GAQRyHAUoKrvwsQ', '2026-11-11 04:13:14+00')
ON CONFLICT DO NOTHING;
";
