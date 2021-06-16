table! {
    atlas (id) {
        id -> Int4,
        tiling_id -> Int4,
        tiling_type_id -> Int4,
    }
}

table! {
    atlasedge (id) {
        id -> Int4,
        source_id -> Int4,
        sink_id -> Int4,
    }
}

table! {
    atlasvertex (id) {
        id -> Int4,
        atlas_id -> Int4,
        title -> Nullable<Varchar>,
    }
}

table! {
    atlasvertexprototile (id) {
        id -> Int4,
        atlas_vertex_id -> Int4,
        polygon_point_id -> Int4,
    }
}

table! {
    label (id) {
        id -> Int4,
        content -> Varchar,
    }
}

table! {
    point (id) {
        id -> Int4,
        x -> Float8,
        y -> Float8,
    }
}

table! {
    polygon (id) {
        id -> Int4,
        title -> Varchar,
    }
}

table! {
    polygonlabel (id) {
        id -> Int4,
        polygon_id -> Int4,
        label_id -> Int4,
    }
}

table! {
    polygonpoint (id) {
        id -> Int4,
        polygon_id -> Int4,
        point_id -> Int4,
        sequence -> Int4,
    }
}

table! {
    tiling (id) {
        id -> Int4,
        title -> Varchar,
        tiling_type_id -> Int4,
    }
}

table! {
    tilinglabel (id) {
        id -> Int4,
        tiling_id -> Int4,
        label_id -> Int4,
    }
}

table! {
    tilingtype (id) {
        id -> Int4,
        title -> Varchar,
    }
}

joinable!(atlas -> tiling (tiling_id));
joinable!(atlasvertex -> atlas (atlas_id));
joinable!(atlasvertexprototile -> atlasvertex (atlas_vertex_id));
joinable!(atlasvertexprototile -> polygonpoint (polygon_point_id));
joinable!(polygonlabel -> label (label_id));
joinable!(polygonlabel -> polygon (polygon_id));
joinable!(polygonpoint -> point (point_id));
joinable!(polygonpoint -> polygon (polygon_id));
joinable!(tiling -> tilingtype (tiling_type_id));
joinable!(tilinglabel -> label (label_id));
joinable!(tilinglabel -> tiling (tiling_id));

allow_tables_to_appear_in_same_query!(
    atlas,
    atlasedge,
    atlasvertex,
    atlasvertexprototile,
    label,
    point,
    polygon,
    polygonlabel,
    polygonpoint,
    tiling,
    tilinglabel,
    tilingtype,
);
