-- Local Postgres setup for vaction_app (run as postgres superuser).
-- Usage:
--   sudo -u postgres psql -v ON_ERROR_STOP=1 -f scripts/setup_postgres.sql

ALTER ROLE vaction WITH LOGIN PASSWORD 'your_password' CREATEDB;

SELECT format('CREATE DATABASE %I OWNER vaction', datname)
FROM (VALUES ('vaction_app'), ('vaction_app_prototype')) AS d(datname)
WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE pg_database.datname = d.datname)\gexec

GRANT ALL PRIVILEGES ON DATABASE vaction_app TO vaction;
GRANT ALL PRIVILEGES ON DATABASE vaction_app_prototype TO vaction;

\c vaction_app
GRANT ALL ON SCHEMA public TO vaction;
ALTER SCHEMA public OWNER TO vaction;

\c vaction_app_prototype
GRANT ALL ON SCHEMA public TO vaction;
ALTER SCHEMA public OWNER TO vaction;
