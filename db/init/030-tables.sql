CREATE TABLE application_entities(
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    title varchar(16) NOT NULL,
    host text NOT NULL,
    port integer NOT NULL CHECK (port >= 1 AND port <= 65535),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

