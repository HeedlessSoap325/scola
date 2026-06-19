CREATE TABLE IF NOT EXISTS student (
    id UUID PRIMARY KEY REFERENCES person(id) ON DELETE CASCADE,
    class_id UUID NOT NULL, FOREIGN KEY(class_id) REFERENCES class(id),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    picture TEXT NOT NULL
);