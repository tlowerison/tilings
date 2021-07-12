CALL delete_full_polygon('6*4π/9');
CALL delete_full_polygon('6*π/2');
CALL delete_full_polygon('6*π/6');
CALL delete_full_polygon('4*π/6');
CALL delete_full_polygon('6*2π/3');

DROP PROCEDURE delete_full_polygon;
DROP PROCEDURE insert_star_polygon;
DROP PROCEDURE insert_regular_polygon;

ALTER TABLE Polygon
  DROP CONSTRAINT polygon_title_key;
