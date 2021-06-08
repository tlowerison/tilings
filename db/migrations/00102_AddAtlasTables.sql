-- +goose Up
CREATE TABLE IF NOT EXISTS Atlas (
  Id        INT  NOT NULL  AUTO_INCREMENT,
  TilingId  INT  NOT NULL,

  PRIMARY KEY (Id),
  FOREIGN KEY (TilingId) REFERENCES Tiling (Id)
);

CREATE TABLE IF NOT EXISTS AtlasVertex (
  Id             INT          NOT NULL  AUTO_INCREMENT,
  AtlasTilingId  INT          NOT NULL,
  Title          VARCHAR(40),

  PRIMARY KEY (Id),
  FOREIGN KEY (AtlasTilingId) REFERENCES Atlas (Id)
);

CREATE TABLE IF NOT EXISTS AtlasVertexProtoTile (
  Id              INT  NOT NULL  AUTO_INCREMENT,
  AtlasVertexId   INT  NOT NULL,
  PolygonPointId  INT  NOT NULL,

  PRIMARY KEY (Id),
  FOREIGN KEY (AtlasVertexId)  REFERENCES AtlasVertex  (Id),
  FOREIGN KEY (PolygonPointId) REFERENCES PolygonPoint (Id)
);

CREATE TABLE IF NOT EXISTS AtlasEdge (
  Id        INT  NOT NULL AUTO_INCREMENT,
  SourceId  INT  NOT NULL,
  SinkId    INT  NOT NULL,

  PRIMARY KEY (Id),
  FOREIGN KEY (SourceId) REFERENCES AtlasVertex (Id),
  FOREIGN KEY (SinkId)   REFERENCES AtlasVertex (Id)
);

-- +goose Down
DROP TABLE IF EXISTS AtlasEdge;
DROP TABLE IF EXISTS AtlasVertexProtoTile;
DROP TABLE IF EXISTS AtlasVertex;
DROP TABLE IF EXISTS Atlas;
