ALTER TABLE Polygon
  ADD UNIQUE (title);

CREATE OR REPLACE PROCEDURE
insert_regular_polygon (title VARCHAR(80), side_length DOUBLE PRECISION, num_sides INT)
LANGUAGE plpgsql AS $$

  DECLARE
    turning_angle DOUBLE PRECISION = 2 * PI() / num_sides;
    point_x       DOUBLE PRECISION = 0.0;
    point_y       DOUBLE PRECISION = 0.0;

  BEGIN
    PERFORM SETVAL('public.point_id_seq', COALESCE(MAX(id), 1)) FROM public.point;
    PERFORM SETVAL('public.polygon_id_seq', COALESCE(MAX(id), 1)) FROM public.polygon;
    PERFORM SETVAL('public.polygonpoint_id_seq', COALESCE(MAX(id), 1)) FROM public.polygonpoint;

    CREATE TEMP TABLE PointId (id INT NOT NULL);

    FOR i IN 0 .. num_sides - 1 LOOP
      WITH point_id_temp AS (
        INSERT INTO Point (x, y) VALUES (point_x, point_y) RETURNING id
      )
      INSERT INTO PointId (id) SELECT * FROM point_id_temp;

      point_x = point_x + (side_length * cos(i * turning_angle));
      point_y = point_y + (side_length * sin(i * turning_angle));
    END LOOP;

    WITH polygon_insert_temp AS (
      INSERT INTO Polygon (title) VALUES (title)
      RETURNING id
    ),

    -- window functions are not allowed in RETURNING
    polygon_temp AS (
      SELECT *, ROW_NUMBER() OVER() AS rn
      FROM polygon_insert_temp
    ),

    point_temp AS (
      SELECT id, ROW_NUMBER() OVER() AS rn
      FROM PointId
      ORDER BY id ASC
    ),

    sequence_temp AS (
      SELECT generate_series AS sequence, ROW_NUMBER() OVER() AS rn
      FROM generate_series(1, num_sides)
    )

    INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
    SELECT polygon_temp.id, point_temp.id, sequence_temp.sequence
    FROM polygon_temp
    JOIN point_temp ON polygon_temp.rn <= point_temp.rn
    JOIN sequence_temp ON point_temp.rn = sequence_temp.rn;

    DROP TABLE PointId;
  END

$$;

CREATE OR REPLACE PROCEDURE
insert_star_polygon (title VARCHAR(80), side_length DOUBLE PRECISION, num_base_sides INT, internal_angle DOUBLE PRECISION)
LANGUAGE plpgsql AS $$

  DECLARE
    convex_angle  DOUBLE PRECISION = PI() - internal_angle;
    concave_angle DOUBLE PRECISION = - (PI() * (1.0 - 2.0 / num_base_sides) - internal_angle);
    point_x       DOUBLE PRECISION = 0.0;
    point_y       DOUBLE PRECISION = 0.0;

  BEGIN
    PERFORM SETVAL('public.point_id_seq', COALESCE(MAX(id), 1)) FROM public.point;
    PERFORM SETVAL('public.polygon_id_seq', COALESCE(MAX(id), 1)) FROM public.polygon;
    PERFORM SETVAL('public.polygonpoint_id_seq', COALESCE(MAX(id), 1)) FROM public.polygonpoint;

    CREATE TEMP TABLE PointId (id INT NOT NULL);

    FOR i IN 0 .. num_base_sides - 1 LOOP
      WITH point_id_temp AS (
        INSERT INTO Point (x, y) VALUES (point_x, point_y) RETURNING id
      )
      INSERT INTO PointId (id) SELECT * FROM point_id_temp;

      point_x = point_x + (side_length * cos(i * convex_angle + i * concave_angle));
      point_y = point_y + (side_length * sin(i * convex_angle + i * concave_angle));

      WITH point_id_temp AS (
        INSERT INTO Point (x, y) VALUES (point_x, point_y) RETURNING id
      )
      INSERT INTO PointId (id) SELECT * FROM point_id_temp;

      point_x = point_x + (side_length * cos(i * convex_angle + (i + 1) * concave_angle));
      point_y = point_y + (side_length * sin(i * convex_angle + (i + 1) * concave_angle));
    END LOOP;

    WITH polygon_insert_temp AS (
      INSERT INTO Polygon (title) VALUES (title)
      RETURNING id
    ),

    -- window functions are not allowed in RETURNING
    polygon_temp AS (
      SELECT *, ROW_NUMBER() OVER() AS rn
      FROM polygon_insert_temp
    ),

    point_temp AS (
      SELECT id, ROW_NUMBER() OVER() AS rn
      FROM PointId
      ORDER BY id ASC
    ),

    sequence_temp AS (
      SELECT generate_series AS sequence, ROW_NUMBER() OVER() AS rn
      FROM generate_series(1, 2 * num_base_sides)
    )

    INSERT INTO PolygonPoint (polygon_id, point_id, sequence)
    SELECT polygon_temp.id, point_temp.id, sequence_temp.sequence
    FROM polygon_temp
    JOIN point_temp ON polygon_temp.rn <= point_temp.rn
    JOIN sequence_temp ON point_temp.rn = sequence_temp.rn;

    DROP TABLE PointId;
  END

$$;

CREATE OR REPLACE PROCEDURE
delete_full_polygon (polygon_title VARCHAR(80))
LANGUAGE plpgsql AS $$

  BEGIN
    CREATE TEMP TABLE PointId (id INT NOT NULL);

    INSERT INTO PointId (id)
    SELECT (point_id)
    FROM PolygonPoint
    JOIN Polygon ON Polygon.id = PolygonPoint.polygon_id
    WHERE Polygon.title = polygon_title;

    DELETE FROM PolygonPoint
    USING Polygon
    WHERE Polygon.id = PolygonPoint.polygon_id
      AND Polygon.title = polygon_title;

    DELETE FROM PolygonLabel
    USING Polygon
    WHERE Polygon.id = PolygonLabel.polygon_id
      AND Polygon.title = polygon_title;

    DELETE FROM Point
    USING PointId
    WHERE PointId.id = Point.id;

    DELETE FROM Polygon
    WHERE Polygon.title = polygon_title;

    DROP TABLE PointId;
  END

$$;

CALL insert_star_polygon('6*2π/3', 1.0, 6, 2 * PI() / 3);
CALL insert_star_polygon('4*π/6', 1.0, 4, PI() / 6);
CALL insert_star_polygon('6*π/6', 1.0, 6, PI() / 6);
CALL insert_star_polygon('6*π/2', 1.0, 6, PI() / 2);
CALL insert_star_polygon('6*4π/9', 1.0, 6, 4 * PI() / 9);

-- irregular
INSERT INTO PolygonLabel (polygon_id, label_id)
SELECT Polygon.id, 10
FROM Polygon
WHERE title IN ('4*π/6', '6*π/6', '6*π/2', '6*4π/9');

-- concave
INSERT INTO PolygonLabel (polygon_id, label_id)
SELECT Polygon.id, 12
FROM Polygon
WHERE title IN ('4*π/6', '6*π/6', '6*π/2', '6*4π/9');

-- star
INSERT INTO PolygonLabel (polygon_id, label_id)
SELECT Polygon.id, 13
FROM Polygon
WHERE title IN ('4*π/6', '6*π/6', '6*π/2', '6*4π/9');
