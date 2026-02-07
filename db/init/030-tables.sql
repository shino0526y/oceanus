CREATE TABLE users(
    uuid uuid NOT NULL DEFAULT uuidv7(),
    id text NOT NULL CHECK (id <> ''),
    name text NOT NULL CHECK (name <> ''),
    role smallint NOT NULL,
    password_hash text NOT NULL CHECK (password_hash <> ''),
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (uuid),
    UNIQUE (id),
    UNIQUE (name)
);

CREATE TABLE users_deleted(
    uuid uuid NOT NULL,
    id text NOT NULL CHECK (id <> ''),
    name text NOT NULL CHECK (name <> ''),
    role smallint NOT NULL,
    password_hash text NOT NULL CHECK (password_hash <> ''),
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL,
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL,
    deleted_by uuid NOT NULL,
    deleted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (uuid)
);

CREATE TABLE login_failure_counts(
    user_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE CASCADE,
    failure_count smallint NOT NULL DEFAULT 0 CHECK (failure_count >= 0),
    last_failure_at timestamptz,
    PRIMARY KEY (user_uuid)
);

CREATE TABLE application_entities(
    uuid uuid NOT NULL DEFAULT uuidv7(),
    title varchar(16) NOT NULL CHECK (title <> ''),
    host text NOT NULL CHECK (host <> ''),
    port integer NOT NULL CHECK (port >= 1 AND port <= 65535),
    comment text NOT NULL DEFAULT '',
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (uuid),
    UNIQUE (title),
    UNIQUE (host, port)
);

CREATE TABLE application_entities_deleted(
    uuid uuid NOT NULL,
    title varchar(16) NOT NULL CHECK (title <> ''),
    host text NOT NULL CHECK (host <> ''),
    port integer NOT NULL CHECK (port >= 1 AND port <= 65535),
    comment text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL,
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL,
    deleted_by uuid NOT NULL,
    deleted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (uuid)
);

CREATE TABLE patients(
    id varchar(16) NOT NULL CHECK (id <> ''),
    name_alphabet varchar(64) NOT NULL,
    name_kanji varchar(64) NOT NULL,
    name_hiragana varchar(64) NOT NULL,
    birth_date date,
    sex smallint NOT NULL CHECK (sex = 0 OR sex = 1 OR sex = 2 OR sex = 9),
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE studies(
    patient_id varchar(16) REFERENCES patients(id),
    instance_uid varchar(64) NOT NULL,
    id varchar(16) NOT NULL,
    study_date date,
    study_time time,
    accession_number varchar(16) NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (instance_uid)
);

CREATE TABLE series(
    study_instance_uid varchar(64) NOT NULL REFERENCES studies(instance_uid),
    instance_uid varchar(64) NOT NULL,
    modality varchar(16) NOT NULL,
    series_number integer,
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (instance_uid)
);

CREATE TABLE sop_instances(
    series_instance_uid varchar(64) NOT NULL REFERENCES series(instance_uid),
    class_uid varchar(64) NOT NULL,
    instance_uid varchar(64) NOT NULL,
    transfer_syntax_uid varchar(64) NOT NULL,
    size integer NOT NULL CHECK (size >= 0 AND size <= 2147483647),
    path text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (instance_uid)
);

