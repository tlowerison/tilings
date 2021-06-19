CREATE TABLE IF NOT EXISTS Role (
  id     SERIAL       PRIMARY KEY,
  title  VARCHAR(60)  NOT NULL
);

CREATE TABLE IF NOT EXISTS Account (
  id            SERIAL        PRIMARY KEY,
  email         VARCHAR(150)  NOT NULL  UNIQUE,
  password      VARCHAR(150)  NOT NULL,
  display_name  VARCHAR(100)  NOT NULL  UNIQUE,
  verified      BOOLEAN       NOT NULL  DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS AccountRole (
  id          SERIAL  PRIMARY KEY,
  account_id  INT     NOT NULL,
  role_id     INT     NOT NULL,

  FOREIGN KEY (account_id) REFERENCES Account (id),
  FOREIGN KEY (role_id)    REFERENCES Role (id)
);

INSERT INTO Role (title)
VALUES
  ('ReadOnly'),
  ('Editor'),
  ('Admin');
