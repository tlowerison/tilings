CREATE TABLE IF NOT EXISTS Label (
  id       SERIAL        PRIMARY KEY,
  content  VARCHAR(160)  NOT NULL  UNIQUE
);

CREATE TABLE IF NOT EXISTS Tiling (
  id     SERIAL       PRIMARY KEY,
  title  VARCHAR(80)  NOT NULL
);

CREATE TABLE IF NOT EXISTS TilingLabel (
  id         SERIAL  PRIMARY KEY,
  tiling_id  INT     NOT NULL,
  label_id   INT     NOT NULL,

  FOREIGN KEY (tiling_id) REFERENCES tiling (Id),
  FOREIGN KEY (label_id)  REFERENCES label  (Id)
);

CREATE TABLE IF NOT EXISTS Polygon (
  id     SERIAL       PRIMARY KEY,
  title  VARCHAR(80)  NOT NULL
);

CREATE TABLE IF NOT EXISTS PolygonLabel (
  id          SERIAL  PRIMARY KEY,
  polygon_id  INT     NOT NULL,
  label_id    INT     NOT NULL,

  FOREIGN KEY (polygon_id) REFERENCES Polygon (id),
  FOREIGN KEY (label_id)   REFERENCES Label   (id)
);

CREATE TABLE IF NOT EXISTS Point (
  id  SERIAL            PRIMARY KEY,
  x   DOUBLE PRECISION  NOT NULL,
  y   DOUBLE PRECISION  NOT NULL
);

CREATE TABLE IF NOT EXISTS PolygonPoint (
  id          SERIAL  PRIMARY KEY,
  polygon_id  INT     NOT NULL,
  point_id    INT     NOT NULL,
  sequence    INT     NOT NULL,

  UNIQUE (polygon_id, sequence),

  FOREIGN KEY (polygon_id) REFERENCES Polygon (id),
  FOREIGN KEY (point_id)   REFERENCES Point   (id)
);
