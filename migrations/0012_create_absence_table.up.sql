DROP TYPE IF EXISTS absence_status;
CREATE TYPE absence_status AS ENUM (
	'pending',
	'rejected',
	'approved'
);

CREATE TABLE IF NOT EXISTS absence (
	id UUID PRIMARY KEY NOT NULL,
	student_id UUID NOT NULL, FOREIGN KEY(student_id) REFERENCES student(id) ON DELETE CASCADE, 
	lesson_id UUID NOT NULL, FOREIGN KEY(lesson_id) REFERENCES lesson(id) ON DELETE CASCADE, 
	reason TEXT NOT NULL,
	status absence_status NOT NULL,
	from_timestamp TIMESTAMPTZ NOT NULL,
	to_timestamp TIMESTAMPTZ NOT NULL
);