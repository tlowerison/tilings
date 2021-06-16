ALTER TABLE Atlas
  DROP CONSTRAINT atlas_tiling_id_tiling_type_id_fkey,
  ADD FOREIGN KEY (tiling_id) REFERENCES Tiling (id),
  DROP CONSTRAINT atlas_tiling_type_id_check,
  DROP COLUMN tiling_type_id
;

ALTER TABLE Tiling
  DROP CONSTRAINT tiling_id_tiling_type_id_key,
  DROP CONSTRAINT tiling_tiling_type_id_fkey,
  DROP COLUMN tiling_type_id
;

DROP TABLE IF EXISTS TilingType;
DROP SEQUENCE IF EXISTS TilingType;
