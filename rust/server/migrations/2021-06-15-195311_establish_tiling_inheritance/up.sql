CREATE TABLE IF NOT EXISTS TilingType (
  Id     SERIAL       PRIMARY KEY,
  Title  VARCHAR(50)  NOT NULL
);

INSERT INTO TilingType (Id, Title)
VALUES
  (1, 'Generic'),
  (2, 'Atlas');

ALTER TABLE Tiling
  ADD COLUMN TilingTypeId INT NOT NULL DEFAULT 1,
  ADD FOREIGN KEY (TilingTypeId) REFERENCES TilingType (Id),
  ADD UNIQUE (Id, TilingTypeId)
;

ALTER TABLE Atlas
  ADD COLUMN TilingTypeId INT NOT NULL DEFAULT 2,
  ADD CHECK (TilingTypeId = 2),
  DROP CONSTRAINT atlas_tilingid_fkey,
  ADD FOREIGN KEY (TilingId, TilingTypeId) REFERENCES Tiling (Id, TilingTypeId)
;
