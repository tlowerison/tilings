table! {
    atlas (id) {
        id -> Int4,
        tilingid -> Int4,
        tilingtypeid -> Int4,
    }
}

table! {
    atlasedge (id) {
        id -> Int4,
        sourceid -> Int4,
        sinkid -> Int4,
    }
}

table! {
    atlasvertex (id) {
        id -> Int4,
        atlasid -> Int4,
        title -> Nullable<Varchar>,
    }
}

table! {
    atlasvertexprototile (id) {
        id -> Int4,
        atlasvertexid -> Int4,
        polygonpointid -> Int4,
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
        polygonid -> Int4,
        labelid -> Int4,
    }
}

table! {
    polygonpoint (id) {
        id -> Int4,
        polygonid -> Int4,
        pointid -> Int4,
        sequence -> Int4,
    }
}

table! {
    tiling (id) {
        id -> Int4,
        title -> Varchar,
        tilingtypeid -> Int4,
    }
}

table! {
    tilinglabel (id) {
        id -> Int4,
        tilingid -> Int4,
        labelid -> Int4,
    }
}

table! {
    tilingtype (id) {
        id -> Int4,
        title -> Varchar,
    }
}

joinable!(atlas -> tiling (tilingid));
joinable!(atlasvertex -> atlas (atlasid));
joinable!(atlasvertexprototile -> atlasvertex (atlasvertexid));
joinable!(atlasvertexprototile -> polygonpoint (polygonpointid));
joinable!(polygonlabel -> label (labelid));
joinable!(polygonlabel -> polygon (polygonid));
joinable!(polygonpoint -> point (pointid));
joinable!(polygonpoint -> polygon (polygonid));
joinable!(tiling -> tilingtype (tilingtypeid));
joinable!(tilinglabel -> label (labelid));
joinable!(tilinglabel -> tiling (tilingid));

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
