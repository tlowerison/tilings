CREATE TABLE IF NOT EXISTS Atlas (
  Id        SERIAL  PRIMARY KEY,
  TilingId  INT     NOT NULL,

  FOREIGN KEY (TilingId) REFERENCES Tiling (Id)
);

CREATE TABLE IF NOT EXISTS AtlasVertex (
  Id       SERIAL       PRIMARY KEY,
  AtlasId  INT          NOT NULL,
  Title    VARCHAR(40),

  FOREIGN KEY (AtlasId) REFERENCES Atlas (Id)
);

CREATE TABLE IF NOT EXISTS AtlasVertexProtoTile (
  Id              SERIAL  PRIMARY KEY,
  AtlasVertexId   INT     NOT NULL,
  PolygonPointId  INT     NOT NULL,

  FOREIGN KEY (AtlasVertexId)  REFERENCES AtlasVertex  (Id),
  FOREIGN KEY (PolygonPointId) REFERENCES PolygonPoint (Id)
);

CREATE TABLE IF NOT EXISTS AtlasEdge (
  Id        SERIAL  PRIMARY KEY,
  SourceId  INT     NOT NULL,
  SinkId    INT     NOT NULL,

  FOREIGN KEY (SourceId) REFERENCES AtlasVertex (Id),
  FOREIGN KEY (SinkId)   REFERENCES AtlasVertex (Id)
);
