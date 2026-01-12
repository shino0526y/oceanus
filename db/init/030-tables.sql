CREATE TABLE users(
    id text PRIMARY KEY,
    name text UNIQUE NOT NULL,
    type smallint NOT NULL CHECK (0 <= type AND type <= 4), -- 0: 管理者, 1: 情シス, 2: 医師, 3: 技師, 4: 事務
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE application_entities(
    title varchar(16) PRIMARY KEY,
    host text NOT NULL,
    port integer NOT NULL CHECK (port >= 1 AND port <= 65535),
    comment text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE patients(
    id varchar(16) PRIMARY KEY,
    name_alphabet varchar(64) NOT NULL,
    name_kanji varchar(64) NOT NULL,
    name_hiragana varchar(64) NOT NULL,
    birth_date date,
    sex smallint NOT NULL CHECK (sex = 0 OR sex = 1 OR sex = 2 OR sex = 9),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE studies(
    patient_id varchar(16) REFERENCES patients(id),
    instance_uid varchar(64) PRIMARY KEY,
    id varchar(16) NOT NULL,
    study_date date,
    study_time time,
    accession_number varchar(16) NOT NULL,
    ae_title varchar(16) NOT NULL REFERENCES application_entities(title),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE series(
    study_instance_uid varchar(64) NOT NULL REFERENCES studies(instance_uid),
    instance_uid varchar(64) PRIMARY KEY,
    modality varchar(16) NOT NULL,
    series_number integer,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE sop_instances(
    series_instance_uid varchar(64) NOT NULL REFERENCES series(instance_uid),
    class_uid varchar(64) NOT NULL,
    instance_uid varchar(64) PRIMARY KEY,
    transfer_syntax_uid varchar(64) NOT NULL,
    size integer NOT NULL CHECK (size >= 0 AND size <= 2147483647),
    path text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

