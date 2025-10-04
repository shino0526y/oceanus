CREATE UNIQUE INDEX uidx_application_entities_title_not_deleted ON application_entities(title)
WHERE
    deleted_at IS NULL;

CREATE UNIQUE INDEX uidx_application_entities_host_port_not_deleted ON application_entities(host, port)
WHERE
    deleted_at IS NULL;

