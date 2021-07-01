table! {
    account (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
        display_name -> Varchar,
        verified -> Bool,
    }
}

table! {
    accountrole (id) {
        id -> Int4,
        account_id -> Int4,
        role_id -> Int4,
    }
}

table! {
    apikey (id) {
        id -> Int4,
        account_id -> Int4,
        content -> Varchar,
    }
}

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
        atlas_id -> Int4,
        polygon_point_id -> Int4,
        source_id -> Int4,
        sink_id -> Int4,
        parity -> Bool,
        sequence -> Int4,
        neighbor_edge_id -> Nullable<Int4>,
    }
}

table! {
    atlasvertex (id) {
        id -> Int4,
        atlas_id -> Int4,
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
    role (id) {
        id -> Int4,
        title -> Varchar,
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

joinable!(accountrole -> account (account_id));
joinable!(accountrole -> role (role_id));
joinable!(apikey -> account (account_id));
joinable!(atlas -> tiling (tiling_id));
joinable!(atlasedge -> atlas (atlas_id));
joinable!(atlasedge -> polygonpoint (polygon_point_id));
joinable!(atlasvertex -> atlas (atlas_id));
joinable!(polygonlabel -> label (label_id));
joinable!(polygonlabel -> polygon (polygon_id));
joinable!(polygonpoint -> point (point_id));
joinable!(polygonpoint -> polygon (polygon_id));
joinable!(tiling -> tilingtype (tiling_type_id));
joinable!(tilinglabel -> label (label_id));
joinable!(tilinglabel -> tiling (tiling_id));

allow_tables_to_appear_in_same_query!(
    account,
    accountrole,
    apikey,
    atlas,
    atlasedge,
    atlasvertex,
    label,
    point,
    polygon,
    polygonlabel,
    polygonpoint,
    role,
    tiling,
    tilinglabel,
    tilingtype,
);
