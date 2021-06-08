-- +goose Up
CREATE TABLE IF NOT EXISTS Tiling (
  Id     INT          NOT NULL  AUTO_INCREMENT,
  Title  VARCHAR(80)  NOT NULL,

  PRIMARY KEY (Id),
  FULLTEXT KEY (Title)
);

CREATE TABLE IF NOT EXISTS TilingLabel (
  Id       INT           NOT NULL  AUTO_INCREMENT,
  Content  VARCHAR(160)  NOT NULL,

  PRIMARY KEY (Id),
  UNIQUE KEY (Content),
  FULLTEXT KEY (Content)
);

CREATE TABLE IF NOT EXISTS TilingToLabel (
  Id             INT  NOT NULL  AUTO_INCREMENT,
  TilingId       INT  NOT NULL,
  TilingLabelId  INT  NOT NULL,

  PRIMARY KEY (Id),
  FOREIGN KEY (TilingId)      REFERENCES Tiling      (Id),
  FOREIGN KEY (TilingLabelId) REFERENCES TilingLabel (Id)
);

CREATE TABLE IF NOT EXISTS Polygon (
  Id     INT          NOT NULL  AUTO_INCREMENT,
  Title  VARCHAR(80)  NOT NULL,

  PRIMARY KEY (Id),
  FULLTEXT KEY (Title)
);

CREATE TABLE IF NOT EXISTS PolygonLabel (
  Id       INT           NOT NULL  AUTO_INCREMENT,
  Content  VARCHAR(160)  NOT NULL,

  PRIMARY KEY (Id),
  UNIQUE KEY (Content),
  FULLTEXT KEY (Content)
);

CREATE TABLE IF NOT EXISTS PolygonToLabel (
  Id              INT  NOT NULL AUTO_INCREMENT,
  PolygonId       INT  NOT NULL,
  PolygonLabelId  INT  NOT NULL,

  PRIMARY KEY (Id),
  FOREIGN KEY (PolygonId)       REFERENCES Polygon      (Id),
  FOREIGN KEY (PolygonLabelId)  REFERENCES PolygonLabel (Id)
);

CREATE TABLE IF NOT EXISTS Point (
  Id         INT     NOT NULL  AUTO_INCREMENT,
  X          DOUBLE  NOT NULL,
  Y          DOUBLE  NOT NULL,

  PRIMARY KEY (Id)
);

CREATE TABLE IF NOT EXISTS PolygonPoint (
  Id         INT  NOT NULL  AUTO_INCREMENT,
  PolygonId  INT  NOT NULL,
  PointId    INT  NOT NULL,
  Sequence   INT  NOT NULL,

  PRIMARY KEY (Id),
  UNIQUE (PolygonId, Sequence ASC)
);

-- +goose Down
DROP TABLE IF EXISTS PolygonPoint;
DROP TABLE IF EXISTS Point;
DROP TABLE IF EXISTS PolygonToLabel;
DROP TABLE IF EXISTS PolygonLabel;
DROP TABLE IF EXISTS Polygon;
DROP TABLE IF EXISTS TilingToLabel;
DROP TABLE IF EXISTS TilingLabel;
DROP TABLE IF EXISTS Label;
DROP TABLE IF EXISTS Tiling;
