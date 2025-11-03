// Final Diesel schema for `db` crate.
// This file is intended to be authoritative for Diesel usage inside `db`.
// If your real Postgres schema differs, prefer regenerating with
// `diesel print-schema > src/schema.rs` from the `db` crate root.

#![allow(dead_code)]

// Table: bot_states
// Fields chosen to match the generated DTOs and models in this crate.
diesel::table! {
    use diesel::sql_types::*;
    bot_states (id) {
        id -> Uuid,
        status_message_id -> Nullable<Int4>,
        target_class_name -> Nullable<Text>,
        untis_school -> Text,
        task_name -> Text,
        untis_login -> Text,
        untis_password -> Text,
        updated_at -> Timestamptz,
        notification_chat_id -> Int8,
        notification_thread_id -> Nullable<Int4>,
        status_chat_id -> Int8,
        status_thread_id -> Nullable<Int4>,
    }
}

// Table: lessons
diesel::table! {
    use diesel::sql_types::*;
    lessons (id) {
        id -> Uuid,
        lesson_id -> Int8,
        date -> Date,
        end_time -> Time,
        lesson_type -> Nullable<Text>,
        start_time -> Time,
        subst_text -> Nullable<Text>,
        lesson_code -> Text,
        classes -> Array<Text>,
        rooms -> Array<Text>,
        subjects -> Array<Text>,
        teachers -> Array<Text>,
        bot_state -> Nullable<Uuid>,
    }
}

// Declare foreign-key relationship: lessons.bot_state -> bot_states.id
diesel::joinable!(lessons -> bot_states (bot_state));

// Allow these tables to appear in the same query (needed for some joins).
diesel::allow_tables_to_appear_in_same_query!(bot_states, lessons);
