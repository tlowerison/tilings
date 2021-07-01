ALTER TABLE AtlasEdge
  DROP CONSTRAINT atlasedge_neighbor_edge_id_fkey,
  DROP COLUMN neighbor_edge_id,
  DROP COLUMN sequence
;
