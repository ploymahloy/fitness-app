PRAGMA foreign_keys = ON;

CREATE TABLE daily_log (
    date TEXT PRIMARY KEY
);

CREATE TABLE nutrition (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    calories INTEGER,
    protein REAL,
    FOREIGN KEY (date) REFERENCES daily_log(date) ON DELETE CASCADE
);

CREATE TABLE cardio_session (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    exercise_name TEXT NOT NULL,
    duration_in_minutes INTEGER,
    after_weight_session INTEGER,
    FOREIGN KEY (date) REFERENCES daily_log(date) ON DELETE CASCADE
);

CREATE TABLE weight_session (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    name TEXT,
    FOREIGN KEY (date) REFERENCES daily_log(date) ON DELETE CASCADE
);

CREATE TABLE exercise (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    weight_session_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (weight_session_id) REFERENCES weight_session(id) ON DELETE CASCADE
);

CREATE TABLE exercise_sets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_id INTEGER NOT NULL,
    weight_in_pounds INTEGER,
    repetitions INTEGER,
    FOREIGN KEY (exercise_id) REFERENCES exercise(id) ON DELETE CASCADE
);