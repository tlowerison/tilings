## DB Design goals
- keep the base Tiling table reduced to common fields and move details into specific enumeration tables (i.e. Atlas)
- Tilings, Polygons, etc. should be queryable by title, label and other properties of interest to support omni search down the road
- tables should be optimized for read operations
