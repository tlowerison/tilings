ALTER TABLE Account
  ADD COLUMN password_reset_code            VARCHAR(150)  UNIQUE,
  ADD COLUMN password_reset_code_timestamp  TIMESTAMP
;
