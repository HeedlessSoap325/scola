DROP TYPE IF EXISTS person_role;
CREATE TYPE person_role AS ENUM (
    'student',
    'teacher',
	'local_admin',
    'admin'
);

CREATE TABLE IF NOT EXISTS person (
    id UUID PRIMARY KEY NOT NULL,
	school_id UUID NOT NULL, FOREIGN KEY(school_id) REFERENCES school(id) ON DELETE CASCADE,
    email VARCHAR(320) NOT NULL,
    login_name VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    role person_role NOT NULL
);