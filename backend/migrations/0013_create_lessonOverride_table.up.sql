DROP TYPE IF EXISTS lesson_status;
CREATE TYPE lesson_status AS ENUM (
	'canceled',
	'moved',
	'room_change',
	'reservation',
	'modified_by_block'
);


CREATE TABLE IF NOT EXISTS "lessonOverride" (
	id UUID PRIMARY KEY NOT NULL,
	lesson_id UUID NOT NULL, FOREIGN KEY(lesson_id) REFERENCES lesson(id) ON DELETE CASCADE, 
	status lesson_status NOT NULL,
	course_id UUID NOT NULL, FOREIGN KEY(course_id) REFERENCES course(id) ON DELETE CASCADE, 
	room_id UUID NOT NULL, FOREIGN KEY(room_id) REFERENCES room(id) ON DELETE CASCADE, 
	start_time TIME NOT NULL,
	end_time TIME NOT NULL,
	title VARCHAR(100) NOT NULL,
	date DATE NOT NULL DEFAULT current_date
);