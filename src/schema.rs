table! {
    locations (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
    }
}

table! {
    entries (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        size -> BigInt,
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
    entry2labels (entry_id, label_id) {
        entry_id -> Integer,
        label_id -> Integer,
    }
}

joinable!(entry2labels -> entries (entry_id));
joinable!(entry2labels -> labels (label_id));

table! {
    location2entries (location_id, entry_id) {
        location_id -> Integer,
        entry_id -> Integer,
    }
}

joinable!(location2entries -> locations (location_id));
joinable!(location2entries -> entries (entry_id));

joinable!(files -> entries (entry_id));

allow_tables_to_appear_in_same_query!(
    entries,
    entry2labels,
    files,
    labels,
);
