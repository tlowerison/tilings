CREATE TABLE IF NOT EXISTS APIKey (
  id          SERIAL        PRIMARY KEY,
  account_id  INT           NOT NULL     UNIQUE,
  content     VARCHAR(256)  NOT NULL     UNIQUE,

  FOREIGN KEY (account_id) REFERENCES Account (id)
);
