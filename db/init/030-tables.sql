CREATE TABLE application_entities(
    title varchar(16) PRIMARY KEY,
    host text NOT NULL,
    port integer NOT NULL CHECK (port >= 1 AND port <= 65535),
    comment text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

