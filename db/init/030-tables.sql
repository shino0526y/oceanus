CREATE TABLE application_entities(
    title varchar(16) PRIMARY KEY,
    host text NOT NULL,
    port integer NOT NULL CHECK (port >= 1 AND port <= 65535),
    comment text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

CREATE TABLE patients(
    id varchar(16) PRIMARY KEY,
    name_alphabet varchar(64) NOT NULL DEFAULT '',
    name_kanji varchar(64) NOT NULL DEFAULT '',
    name_hiragana varchar(64) NOT NULL DEFAULT '',
    birth_date date,
    sex smallint CHECK (sex >= 0 AND sex <= 2),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

CREATE TABLE studies(
    patient_id varchar(16) REFERENCES patients(id),
    instance_uid varchar(64) PRIMARY KEY,
    id varchar(16) NOT NULL DEFAULT '',
    study_date date,
    study_time time,
    accession_number varchar(16) NOT NULL DEFAULT '',
    ae_title varchar(16) NOT NULL REFERENCES application_entities(title),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

CREATE TABLE series(
    study_instance_uid varchar(64) NOT NULL REFERENCES studies(instance_uid),
    instance_uid varchar(64) PRIMARY KEY,
    modality varchar(16) NOT NULL DEFAULT '',
    series_number integer,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

CREATE TABLE sop_instances(
    series_instance_uid varchar(64) NOT NULL REFERENCES series(instance_uid),
    class_uid varchar(64) NOT NULL DEFAULT '',
    instance_uid varchar(64) PRIMARY KEY,
    path text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz DEFAULT NULL
);

