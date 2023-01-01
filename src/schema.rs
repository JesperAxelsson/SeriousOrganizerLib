// @generated automatically by Diesel CLI.

diesel::table! {
    entries (id) {
        id -> Integer,
        location_id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
        grade -> Nullable<Integer>,
    }
}

diesel::table! {
    entry2labels (entry_id, label_id) {
        entry_id -> Integer,
        label_id -> Integer,
    }
}

diesel::table! {
    files (id) {
        id -> Integer,
        entry_id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
    }
}

diesel::table! {
    label_auto_filters (id) {
        id -> Integer,
        name -> Text,
        filter -> Text,
        label_id -> Integer,
    }
}

diesel::table! {
    labels (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    locations (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
    }
}

diesel::joinable!(entries -> locations (location_id));
diesel::joinable!(entry2labels -> entries (entry_id));
diesel::joinable!(entry2labels -> labels (label_id));
diesel::joinable!(files -> entries (entry_id));
diesel::joinable!(label_auto_filters -> labels (label_id));

diesel::allow_tables_to_appear_in_same_query!(
    entries,
    entry2labels,
    files,
    label_auto_filters,
    labels,
    locations,
);
