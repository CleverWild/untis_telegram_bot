// @generated automatically by Diesel CLI.

diesel::table! {
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
        timezone -> Text,
    }
}

diesel::table! {
    lessons (lesson_id) {
        bot_state -> Uuid,
        lesson_id -> Int8,
        date -> Date,
        end_time -> Time,
        lesson_type -> Text,
        start_time -> Time,
        subst_text -> Nullable<Text>,
        lesson_code -> Text,
        classes -> Array<Text>,
        rooms -> Array<Text>,
        subjects -> Array<Text>,
        teachers -> Array<Text>,
    }
}

diesel::joinable!(lessons -> bot_states (bot_state));

diesel::allow_tables_to_appear_in_same_query!(bot_states, lessons,);
