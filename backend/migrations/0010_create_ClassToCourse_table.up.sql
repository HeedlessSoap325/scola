CREATE TABLE IF NOT EXISTS "classToCourse" (
	class_id UUID NOT NULL, FOREIGN KEY(class_id) REFERENCES class(id) ON DELETE CASCADE, 
	course_id UUID NOT NULL, FOREIGN KEY(course_id) REFERENCES course(id) ON DELETE CASCADE, 
	semester_id UUID NOT NULL, FOREIGN KEY(semester_id) REFERENCES semester(id) ON DELETE CASCADE,
	PRIMARY KEY (class_id, course_id, semester_id) 
);