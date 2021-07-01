DELETE FROM AtlasEdge;
DELETE FROM AtlasVertex;
DELETE FROM Atlas;

DELETE FROM TilingLabel tl
  USING Tiling t
  WHERE tl.tiling_id = t.id
    AND t.tiling_type_id = 2;

DELETE FROM Tiling t
  WHERE t.tiling_type_id = 2;

ALTER TABLE AtlasEdge
  ADD COLUMN sequence         INT  NOT NULL,
  ADD COLUMN neighbor_edge_id INT,
  ADD FOREIGN KEY (neighbor_edge_id) REFERENCES AtlasEdge (id)
;
