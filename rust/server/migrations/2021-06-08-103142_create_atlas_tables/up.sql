CREATE TABLE IF NOT EXISTS Atlas (
  id         SERIAL  PRIMARY KEY,
  tiling_id  INT     NOT NULL,

  FOREIGN KEY (tiling_id) REFERENCES Tiling (id)
);

CREATE TABLE IF NOT EXISTS AtlasVertex (
  id        SERIAL       PRIMARY KEY,
  atlas_id  INT          NOT NULL,
  title     VARCHAR(40),

  FOREIGN KEY (atlas_id) REFERENCES Atlas (id)
);

CREATE TABLE IF NOT EXISTS AtlasEdge (
  id                SERIAL  PRIMARY KEY,
  atlas_id          INT     NOT NULL,
  polygon_point_id  INT     NOT NULL, -- point of the polygon included in the source vertex
  source_id         INT     NOT NULL,
  sink_id           INT     NOT NULL,

  FOREIGN KEY (atlas_id)         REFERENCES Atlas (id),
  FOREIGN KEY (polygon_point_id) REFERENCES PolygonPoint (id),
  FOREIGN KEY (source_id)        REFERENCES AtlasVertex (id),
  FOREIGN KEY (sink_id)          REFERENCES AtlasVertex (id)
);
