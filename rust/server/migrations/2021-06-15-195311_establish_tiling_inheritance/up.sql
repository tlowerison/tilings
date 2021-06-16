CREATE TABLE IF NOT EXISTS TilingType (
  id     SERIAL       PRIMARY KEY,
  title  VARCHAR(50)  NOT NULL
);

INSERT INTO TilingType (id, title)
VALUES
  (1, 'Generic'),
  (2, 'Atlas');

ALTER TABLE Tiling
  ADD COLUMN tiling_type_id INT NOT NULL DEFAULT 1,
  ADD FOREIGN KEY (tiling_type_id) REFERENCES TilingType (id),
  ADD UNIQUE (id, tiling_type_id)
;

ALTER TABLE Atlas
  ADD COLUMN tiling_type_id INT NOT NULL DEFAULT 2,
  ADD CHECK (tiling_type_id = 2),
  DROP CONSTRAINT atlas_tiling_id_fkey,
  ADD FOREIGN KEY (tiling_id, tiling_type_id) REFERENCES Tiling (id, tiling_type_id)
;
