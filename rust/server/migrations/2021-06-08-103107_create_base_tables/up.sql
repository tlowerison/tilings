CREATE TABLE IF NOT EXISTS Label (
  id       SERIAL        PRIMARY KEY,
  content  VARCHAR(160)  NOT NULL  UNIQUE
);

CREATE TABLE IF NOT EXISTS Tiling (
  id     SERIAL       PRIMARY KEY,
  title  VARCHAR(80)  NOT NULL
);

CREATE TABLE IF NOT EXISTS TilingLabel (
  id         SERIAL  PRIMARY KEY,
  tiling_id  INT     NOT NULL,
  label_id   INT     NOT NULL,

  FOREIGN KEY (tiling_id) REFERENCES tiling (Id),
  FOREIGN KEY (label_id)  REFERENCES label  (Id)
);

CREATE TABLE IF NOT EXISTS Polygon (
  id     SERIAL       PRIMARY KEY,
  title  VARCHAR(80)  NOT NULL
);

CREATE TABLE IF NOT EXISTS PolygonLabel (
  id          SERIAL  PRIMARY KEY,
  polygon_id  INT     NOT NULL,
  label_id    INT     NOT NULL,

  FOREIGN KEY (polygon_id) REFERENCES Polygon (id),
  FOREIGN KEY (label_id)   REFERENCES Label   (id)
);

CREATE TABLE IF NOT EXISTS Point (
  id  SERIAL            PRIMARY KEY,
  x   DOUBLE PRECISION  NOT NULL,
  y   DOUBLE PRECISION  NOT NULL
);

CREATE TABLE IF NOT EXISTS PolygonPoint (
  id          SERIAL  PRIMARY KEY,
  polygon_id  INT     NOT NULL,
  point_id    INT     NOT NULL,
  sequence    INT     NOT NULL,

  UNIQUE (polygon_id, sequence),

  FOREIGN KEY (polygon_id) REFERENCES Polygon (id),
  FOREIGN KEY (point_id)   REFERENCES Point   (id)
);

INSERT INTO Label (id, content)
VALUES
  (1,  '1-uniform'),
  (2,  '2-uniform'),
  (3,  '3-uniform'),
  (4,  '4-uniform'),
  (5,  '5-uniform'),
  (6,  '6-uniform'),
  (7,  'edge-to-edge'),
  (8,  'non-edge-to-edge'),
  (9,  'regular'),
  (10, 'irregular'),
  (11, 'convex'),
  (12, 'concave'),
  (13, 'star');

INSERT INTO Polygon (id, title)
  VALUES
  (1, 'Unit Triangle'),
  (2, 'Unit Square'),
  (3, 'Unit Pentagon'),
  (4, 'Unit Hexagon'),
  (5, 'Unit Heptagon'),
  (6, 'Unit Octagon'),
  (7, 'Unit Nonagon'),
  (8, 'Unit Decagon'),
  (9, 'Unit Dodecagon');

INSERT INTO PolygonLabel (polygon_id, label_id)
  VALUES
  (1, 9),
  (1, 11),
  (2, 9),
  (2, 11),
  (3, 9),
  (3, 11),
  (4, 9),
  (4, 11),
  (5, 9),
  (5, 11),
  (6, 9),
  (6, 11),
  (7, 9),
  (7, 11),
  (8, 9),
  (8, 11),
  (9, 9),
  (9, 11);

-- Unit Triangle
INSERT INTO Point (id, x, y)
  VALUES
  (
    1,
    0,
    0
  ),
  (
    2,
    1,
    0
  ),
  (
    3,
    1 + cos(2 * 2 * pi() / 6),
    0 + sin(2 * 2 * pi() / 6)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (1, 1, 1),
  (1, 2, 2),
  (1, 3, 3);

-- Unit Square
INSERT INTO Point (id, x, y)
  VALUES
  (
    4,
    0,
    0
  ),
  (
    5,
    1,
    0
  ),
  (
    6,
    1,
    1
  ),
  (
    7,
    0,
    1
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (2, 4, 1),
  (2, 5, 2),
  (2, 6, 3),
  (2, 7, 4);

-- Unit Pentagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    8,
    0,
    0
  ),
  (
    9,
    1,
    0
  ),
  (
    10,
    1 + cos(1 * 2 * pi() / 5),
    0 + sin(1 * 2 * pi() / 5)
  ),
  (
    11,
    1 + cos(1 * 2 * pi() / 5) + cos(2 * 2 * pi() / 5),
    0 + sin(1 * 2 * pi() / 5) + sin(2 * 2 * pi() / 5)
  ),
  (
    12,
    1 + cos(1 * 2 * pi() / 5) + cos(2 * 2 * pi() / 5) + cos(3 * 2 * pi() / 5),
    0 + sin(1 * 2 * pi() / 5) + sin(2 * 2 * pi() / 5) + sin(3 * 2 * pi() / 5)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (3, 8,  1),
  (3, 9,  2),
  (3, 10, 3),
  (3, 11, 4),
  (3, 12, 5);

-- Unit Hexagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    13,
    0,
    0
  ),
  (
    14,
    1,
    0
  ),
  (
    15,
    1 + cos(1 * 2 * pi() / 6),
    0 + sin(1 * 2 * pi() / 6)
  ),
  (
    16,
    1 + cos(1 * 2 * pi() / 6) + cos(2 * 2 * pi() / 6),
    0 + sin(1 * 2 * pi() / 6) + sin(2 * 2 * pi() / 6)
  ),
  (
    17,
    1 + cos(1 * 2 * pi() / 6) + cos(2 * 2 * pi() / 6) + cos(3 * 2 * pi() / 6),
    0 + sin(1 * 2 * pi() / 6) + sin(2 * 2 * pi() / 6) + sin(3 * 2 * pi() / 6)
  ),
  (
    18,
    1 + cos(1 * 2 * pi() / 6) + cos(2 * 2 * pi() / 6) + cos(3 * 2 * pi() / 6) + cos(4 * 2 * pi() / 6),
    0 + sin(1 * 2 * pi() / 6) + sin(2 * 2 * pi() / 6) + sin(3 * 2 * pi() / 6) + sin(4 * 2 * pi() / 6)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (4, 13, 1),
  (4, 14, 2),
  (4, 15, 3),
  (4, 16, 4),
  (4, 17, 5),
  (4, 18, 6);

-- Unit Heptagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    19,
    0,
    0
  ),
  (
    20,
    1,
    0
  ),
  (
    21,
    1 + cos(1 * 2 * pi() / 7),
    0 + sin(1 * 2 * pi() / 7)
  ),
  (
    22,
    1 + cos(1 * 2 * pi() / 7) + cos(2 * 2 * pi() / 7),
    0 + sin(1 * 2 * pi() / 7) + sin(2 * 2 * pi() / 7)
  ),
  (
    23,
    1 + cos(1 * 2 * pi() / 7) + cos(2 * 2 * pi() / 7) + cos(3 * 2 * pi() / 7),
    0 + sin(1 * 2 * pi() / 7) + sin(2 * 2 * pi() / 7) + sin(3 * 2 * pi() / 7)
  ),
  (
    24,
    1 + cos(1 * 2 * pi() / 7) + cos(2 * 2 * pi() / 7) + cos(3 * 2 * pi() / 7) + cos(4 * 2 * pi() / 7),
    0 + sin(1 * 2 * pi() / 7) + sin(2 * 2 * pi() / 7) + sin(3 * 2 * pi() / 7) + sin(4 * 2 * pi() / 7)
  ),
  (
    25,
    1 + cos(1 * 2 * pi() / 7) + cos(2 * 2 * pi() / 7) + cos(3 * 2 * pi() / 7) + cos(4 * 2 * pi() / 7) + cos(5 * 2 * pi() / 7),
    0 + sin(1 * 2 * pi() / 7) + sin(2 * 2 * pi() / 7) + sin(3 * 2 * pi() / 7) + sin(4 * 2 * pi() / 7) + sin(5 * 2 * pi() / 7)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (5, 19, 1),
  (5, 20, 2),
  (5, 21, 3),
  (5, 22, 4),
  (5, 23, 5),
  (5, 24, 6),
  (5, 25, 7);

-- Unit Octagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    26,
    0,
    0
  ),
  (
    27,
    1,
    0
  ),
  (
    28,
    1 + cos(1 * 2 * pi() / 8),
    0 + sin(1 * 2 * pi() / 8)
  ),
  (
    29,
    1 + cos(1 * 2 * pi() / 8) + cos(2 * 2 * pi() / 8),
    0 + sin(1 * 2 * pi() / 8) + sin(2 * 2 * pi() / 8)
  ),
  (
    30,
    1 + cos(1 * 2 * pi() / 8) + cos(2 * 2 * pi() / 8) + cos(3 * 2 * pi() / 8),
    0 + sin(1 * 2 * pi() / 8) + sin(2 * 2 * pi() / 8) + sin(3 * 2 * pi() / 8)
  ),
  (
    31,
    1 + cos(1 * 2 * pi() / 8) + cos(2 * 2 * pi() / 8) + cos(3 * 2 * pi() / 8) + cos(4 * 2 * pi() / 8),
    0 + sin(1 * 2 * pi() / 8) + sin(2 * 2 * pi() / 8) + sin(3 * 2 * pi() / 8) + sin(4 * 2 * pi() / 8)
  ),
  (
    32,
    1 + cos(1 * 2 * pi() / 8) + cos(2 * 2 * pi() / 8) + cos(3 * 2 * pi() / 8) + cos(4 * 2 * pi() / 8) + cos(5 * 2 * pi() / 8),
    0 + sin(1 * 2 * pi() / 8) + sin(2 * 2 * pi() / 8) + sin(3 * 2 * pi() / 8) + sin(4 * 2 * pi() / 8) + sin(5 * 2 * pi() / 8)
  ),
  (
    33,
    1 + cos(1 * 2 * pi() / 8) + cos(2 * 2 * pi() / 8) + cos(3 * 2 * pi() / 8) + cos(4 * 2 * pi() / 8) + cos(5 * 2 * pi() / 8) + cos(6 * 2 * pi() / 8),
    0 + sin(1 * 2 * pi() / 8) + sin(2 * 2 * pi() / 8) + sin(3 * 2 * pi() / 8) + sin(4 * 2 * pi() / 8) + sin(5 * 2 * pi() / 8) + sin(6 * 2 * pi() / 8)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (6, 26, 1),
  (6, 27, 2),
  (6, 28, 3),
  (6, 29, 4),
  (6, 30, 5),
  (6, 31, 6),
  (6, 32, 7),
  (6, 33, 8);

-- Unit Nonagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    34,
    0,
    0
  ),
  (
    35,
    1,
    0
  ),
  (
    36,
    1 + cos(1 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9)
  ),
  (
    37,
    1 + cos(1 * 2 * pi() / 9) + cos(2 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9) + sin(2 * 2 * pi() / 9)
  ),
  (
    38,
    1 + cos(1 * 2 * pi() / 9) + cos(2 * 2 * pi() / 9) + cos(3 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9) + sin(2 * 2 * pi() / 9) + sin(3 * 2 * pi() / 9)
  ),
  (
    39,
    1 + cos(1 * 2 * pi() / 9) + cos(2 * 2 * pi() / 9) + cos(3 * 2 * pi() / 9) + cos(4 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9) + sin(2 * 2 * pi() / 9) + sin(3 * 2 * pi() / 9) + sin(4 * 2 * pi() / 9)
  ),
  (
    40,
    1 + cos(1 * 2 * pi() / 9) + cos(2 * 2 * pi() / 9) + cos(3 * 2 * pi() / 9) + cos(4 * 2 * pi() / 9) + cos(5 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9) + sin(2 * 2 * pi() / 9) + sin(3 * 2 * pi() / 9) + sin(4 * 2 * pi() / 9) + sin(5 * 2 * pi() / 9)
  ),
  (
    41,
    1 + cos(1 * 2 * pi() / 9) + cos(2 * 2 * pi() / 9) + cos(3 * 2 * pi() / 9) + cos(4 * 2 * pi() / 9) + cos(5 * 2 * pi() / 9) + cos(6 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9) + sin(2 * 2 * pi() / 9) + sin(3 * 2 * pi() / 9) + sin(4 * 2 * pi() / 9) + sin(5 * 2 * pi() / 9) + sin(6 * 2 * pi() / 9)
  ),
  (
    42,
    1 + cos(1 * 2 * pi() / 9) + cos(2 * 2 * pi() / 9) + cos(3 * 2 * pi() / 9) + cos(4 * 2 * pi() / 9) + cos(5 * 2 * pi() / 9) + cos(6 * 2 * pi() / 9) + cos(7 * 2 * pi() / 9),
    0 + sin(1 * 2 * pi() / 9) + sin(2 * 2 * pi() / 9) + sin(3 * 2 * pi() / 9) + sin(4 * 2 * pi() / 9) + sin(5 * 2 * pi() / 9) + sin(6 * 2 * pi() / 9) + sin(7 * 2 * pi() / 9)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (7, 34, 1),
  (7, 35, 2),
  (7, 36, 3),
  (7, 37, 4),
  (7, 38, 5),
  (7, 39, 6),
  (7, 40, 7),
  (7, 41, 8),
  (7, 42, 9);

-- Unit Decagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    43,
    0,
    0
  ),
  (
    44,
    1,
    0
  ),
  (
    45,
    1 + cos(1 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10)
  ),
  (
    46,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10)
  ),
  (
    47,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10) + cos(3 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10) + sin(3 * 2 * pi() / 10)
  ),
  (
    48,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10) + cos(3 * 2 * pi() / 10) + cos(4 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10) + sin(3 * 2 * pi() / 10) + sin(4 * 2 * pi() / 10)
  ),
  (
    49,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10) + cos(3 * 2 * pi() / 10) + cos(4 * 2 * pi() / 10) + cos(5 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10) + sin(3 * 2 * pi() / 10) + sin(4 * 2 * pi() / 10) + sin(5 * 2 * pi() / 10)
  ),
  (
    50,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10) + cos(3 * 2 * pi() / 10) + cos(4 * 2 * pi() / 10) + cos(5 * 2 * pi() / 10) + cos(6 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10) + sin(3 * 2 * pi() / 10) + sin(4 * 2 * pi() / 10) + sin(5 * 2 * pi() / 10) + sin(6 * 2 * pi() / 10)
  ),
  (
    51,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10) + cos(3 * 2 * pi() / 10) + cos(4 * 2 * pi() / 10) + cos(5 * 2 * pi() / 10) + cos(6 * 2 * pi() / 10) + cos(7 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10) + sin(3 * 2 * pi() / 10) + sin(4 * 2 * pi() / 10) + sin(5 * 2 * pi() / 10) + sin(6 * 2 * pi() / 10) + sin(7 * 2 * pi() / 10)
  ),
  (
    52,
    1 + cos(1 * 2 * pi() / 10) + cos(2 * 2 * pi() / 10) + cos(3 * 2 * pi() / 10) + cos(4 * 2 * pi() / 10) + cos(5 * 2 * pi() / 10) + cos(6 * 2 * pi() / 10) + cos(7 * 2 * pi() / 10) + cos(8 * 2 * pi() / 10),
    0 + sin(1 * 2 * pi() / 10) + sin(2 * 2 * pi() / 10) + sin(3 * 2 * pi() / 10) + sin(4 * 2 * pi() / 10) + sin(5 * 2 * pi() / 10) + sin(6 * 2 * pi() / 10) + sin(7 * 2 * pi() / 10) + sin(8 * 2 * pi() / 10)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (8, 43, 1),
  (8, 44, 2),
  (8, 45, 3),
  (8, 46, 4),
  (8, 47, 5),
  (8, 48, 6),
  (8, 49, 7),
  (8, 50, 8),
  (8, 51, 9),
  (8, 52, 10);

-- Unit Dodecagon
INSERT INTO Point (id, x, y)
  VALUES
  (
    53,
    0,
    0
  ),
  (
    54,
    1,
    0
  ),
  (
    55,
    1 + cos(1 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12)
  ),
  (
    56,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12)
  ),
  (
    57,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12)
  ),
  (
    58,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12)
  ),
  (
    59,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12) + cos(5 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12) + sin(5 * 2 * pi() / 12)
  ),
  (
    60,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12) + cos(5 * 2 * pi() / 12) + cos(6 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12) + sin(5 * 2 * pi() / 12) + sin(6 * 2 * pi() / 12)
  ),
  (
    61,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12) + cos(5 * 2 * pi() / 12) + cos(6 * 2 * pi() / 12) + cos(7 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12) + sin(5 * 2 * pi() / 12) + sin(6 * 2 * pi() / 12) + sin(7 * 2 * pi() / 12)
  ),
  (
    62,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12) + cos(5 * 2 * pi() / 12) + cos(6 * 2 * pi() / 12) + cos(7 * 2 * pi() / 12) + cos(8 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12) + sin(5 * 2 * pi() / 12) + sin(6 * 2 * pi() / 12) + sin(7 * 2 * pi() / 12) + sin(8 * 2 * pi() / 12)
  ),
  (
    63,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12) + cos(5 * 2 * pi() / 12) + cos(6 * 2 * pi() / 12) + cos(7 * 2 * pi() / 12) + cos(8 * 2 * pi() / 12) + cos(9 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12) + sin(5 * 2 * pi() / 12) + sin(6 * 2 * pi() / 12) + sin(7 * 2 * pi() / 12) + sin(8 * 2 * pi() / 12) + sin(9 * 2 * pi() / 12)
  ),
  (
    64,
    1 + cos(1 * 2 * pi() / 12) + cos(2 * 2 * pi() / 12) + cos(3 * 2 * pi() / 12) + cos(4 * 2 * pi() / 12) + cos(5 * 2 * pi() / 12) + cos(6 * 2 * pi() / 12) + cos(7 * 2 * pi() / 12) + cos(8 * 2 * pi() / 12) + cos(9 * 2 * pi() / 12) + cos(10 * 2 * pi() / 12),
    0 + sin(1 * 2 * pi() / 12) + sin(2 * 2 * pi() / 12) + sin(3 * 2 * pi() / 12) + sin(4 * 2 * pi() / 12) + sin(5 * 2 * pi() / 12) + sin(6 * 2 * pi() / 12) + sin(7 * 2 * pi() / 12) + sin(8 * 2 * pi() / 12) + sin(9 * 2 * pi() / 12) + sin(10 * 2 * pi() / 12)
  );

INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
  VALUES
  (9, 53, 1),
  (9, 54, 2),
  (9, 55, 3),
  (9, 56, 4),
  (9, 57, 5),
  (9, 58, 6),
  (9, 59, 7),
  (9, 60, 8),
  (9, 61, 9),
  (9, 62, 10),
  (9, 63, 11),
  (9, 64, 12);
