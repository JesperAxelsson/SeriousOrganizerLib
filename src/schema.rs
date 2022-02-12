table! {
    entries (id) {
        id -> Integer,
        location_id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
    }
}

table! {
    entry2labels (entry_id, label_id) {
        entry_id -> Integer,
        label_id -> Integer,
    }
}

table! {
    files (id) {
        id -> Integer,
        entry_id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
    }
}

table! {
    labels (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    locations (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
    }
}

table! {
    label_auto_filters (id) {
        id -> Integer,
        name -> Text,
        filter -> Text,
        label_id -> Integer,
    }
}

joinable!(entries -> locations (location_id));
joinable!(entry2labels -> entries (entry_id));
joinable!(entry2labels -> labels (label_id));
joinable!(files -> entries (entry_id));
joinable!(label_auto_filters -> labels (label_id));

allow_tables_to_appear_in_same_query!(
    entries,
    entry2labels,
    files,
    labels,
    locations,
    label_auto_filters
);
