ALTER TABLE Atlas
  DROP CONSTRAINT atlas_tilingid_tilingtypeid_fkey,
  ADD FOREIGN KEY (TilingId) REFERENCES Tiling (Id),
  DROP CONSTRAINT atlas_tilingtypeid_check,
  DROP COLUMN TilingTypeId
;

ALTER TABLE Tiling
  DROP CONSTRAINT tiling_id_tilingtypeid_key,
  DROP CONSTRAINT tiling_tilingtypeid_fkey,
  DROP COLUMN TilingTypeId
;

DROP TABLE IF EXISTS TilingType;
