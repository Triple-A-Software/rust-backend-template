-- Add migration script here
create schema if not exists auth;

create table if not exists auth.user (
    id uuid primary key default gen_random_uuid() not null,
    email text not null,
    first_name text,
    last_name text,
    salt bytea not null,
    hash bytea not null,
    language text DEFAULT 'en' NOT NULL,
    location text,
    description text,
    title text,
    role text DEFAULT 'author' NOT NULL,
    theme text DEFAULT 'light' NOT NULL,
    avatar text,
    online_status text DEFAULT 'offline' NOT NULL,
    last_active_at timestamptz,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    deleted_at timestamptz,
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    deleted_by uuid,
    CONSTRAINT "user_email_deleted_at_unique" UNIQUE NULLS NOT DISTINCT("email","deleted_at")
);

create table if not exists auth.session (
    id serial PRIMARY KEY NOT NULL,
    token_id integer NOT NULL,
    ip_address inet NOT NULL,
    user_agent text NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE IF NOT EXISTS tag (
    id serial PRIMARY KEY NOT NULL,
    title text NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    deleted_at timestamptz,
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    deleted_by uuid
);

CREATE TABLE IF NOT EXISTS auth.user_to_tag (
    user_id uuid NOT NULL,
    tag_id integer NOT NULL,

    CONSTRAINT user_to_tag_user_id_fk
        FOREIGN KEY (user_id)
        REFERENCES auth.user(id),

    CONSTRAINT user_to_tag_tag_id_fk
        FOREIGN KEY (tag_id)
        REFERENCES tag(id)
);

create table if not exists auth.token (
    id serial PRIMARY KEY NOT NULL,
    name text,
    token text NOT NULL,
    token_type text NOT NULL,
    expiration timestamptz,
    user_id uuid NOT NULL,
    session_id integer,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,

    CONSTRAINT token_user_id_fk
        FOREIGN KEY (user_id)
        REFERENCES auth.user(id),

    CONSTRAINT token_session_id_fk
        FOREIGN KEY (session_id)
        REFERENCES auth.session(id)
);

CREATE OR REPLACE FUNCTION update_updated_at_function()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = clock_timestamp();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS settings (
    id text PRIMARY KEY DEFAULT 'settings' NOT NULL,
    setup_finished boolean DEFAULT false NOT NULL
);

CREATE TABLE IF NOT EXISTS activity (
    id serial PRIMARY KEY NOT NULL,
    action text NOT NULL,
    action_by_id uuid NOT NULL,
    action_at timestamptz DEFAULT now() NOT NULL,
    ip_address inet,
    user_agent text,
    table_name text,
    item_id text,
    old_data text,
    new_data text,

    CONSTRAINT action_by_id_user
        FOREIGN KEY (action_by_id)
        REFERENCES auth.user(id)
);
