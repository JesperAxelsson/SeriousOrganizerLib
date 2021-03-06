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

joinable!(entries -> locations (location_id));
joinable!(entry2labels -> entries (entry_id));
joinable!(entry2labels -> labels (label_id));
joinable!(files -> entries (entry_id));

allow_tables_to_appear_in_same_query!(entries, entry2labels, files, labels, locations,);
