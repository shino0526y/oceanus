ARG INCLUDE_DEV_DATA=false

FROM postgres:18-alpine AS base
COPY db/init /docker-entrypoint-initdb.d

FROM base AS dev-data-true
COPY data/db /docker-entrypoint-initdb.d

FROM base AS dev-data-false

FROM dev-data-${INCLUDE_DEV_DATA}
