ALTER TABLE Tiling
  ADD COLUMN owner_id  INT,
  ADD FOREIGN KEY (owner_id) REFERENCES Account (id)
;

ALTER TABLE Polygon
  ADD COLUMN owner_id  INT,
  ADD FOREIGN KEY (owner_id) REFERENCES Account (id)
;
