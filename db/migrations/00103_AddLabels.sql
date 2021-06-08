-- +goose Up
INSERT INTO TilingLabel (Id, Content)
VALUES
  (1, '1-uniform'),
  (2, '2-uniform'),
  (3, '3-uniform'),
  (4, '4-uniform'),
  (5, '5-uniform'),
  (6, '6-uniform'),
  (7, 'edge-to-edge'),
  (8, 'non-edge-to-edge');

INSERT INTO PolygonLabel (Id, Content)
VALUES
  (1, 'regular'),
  (2, 'irregular'),
  (3, 'convex'),
  (4, 'concave'),
  (5, 'star');

-- +goose Down
DELETE FROM PolygonLabel WHERE Id >= 1 AND Id <= 5;
DELETE FROM TilingLabel  WHERE Id >= 1 AND Id <= 8;
