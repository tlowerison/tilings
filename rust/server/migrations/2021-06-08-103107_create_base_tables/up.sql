CREATE TABLE IF NOT EXISTS Label (
  Id       SERIAL                  PRIMARY KEY,
  Content  VARCHAR(160)  NOT NULL  UNIQUE
);

CREATE TABLE IF NOT EXISTS Tiling (
  Id     SERIAL       PRIMARY KEY,
  Title  VARCHAR(80)  NOT NULL
);

CREATE TABLE IF NOT EXISTS TilingLabel (
  Id        SERIAL            PRIMARY KEY,
  TilingId  INT     NOT NULL,
  LabelId   INT     NOT NULL,

  FOREIGN KEY (TilingId) REFERENCES Tiling (Id),
  FOREIGN KEY (LabelId)  REFERENCES Label  (Id)
);

CREATE TABLE IF NOT EXISTS Polygon (
  Id     SERIAL                 PRIMARY KEY,
  Title  VARCHAR(80)  NOT NULL
);

CREATE TABLE IF NOT EXISTS PolygonLabel (
  Id         SERIAL            PRIMARY KEY,
  PolygonId  INT     NOT NULL,
  LabelId    INT     NOT NULL,

  FOREIGN KEY (PolygonId) REFERENCES Polygon (Id),
  FOREIGN KEY (LabelId)   REFERENCES Label   (Id)
);

CREATE TABLE IF NOT EXISTS Point (
  Id         SERIAL                      PRIMARY KEY,
  X          DOUBLE PRECISION  NOT NULL,
  Y          DOUBLE PRECISION  NOT NULL
);

CREATE TABLE IF NOT EXISTS PolygonPoint (
  Id         SERIAL            PRIMARY KEY,
  PolygonId  INT     NOT NULL,
  PointId    INT     NOT NULL,
  Sequence   INT     NOT NULL,

  UNIQUE (PolygonId, Sequence),

  FOREIGN KEY (PolygonId) REFERENCES Polygon (Id),
  FOREIGN KEY (PointId)   REFERENCES Point   (Id)
);
