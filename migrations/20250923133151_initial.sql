-- Add migration script here

PRAGMA foreign_keys = ON;

CREATE TABLE users (
    discord_id INTEGER PRIMARY KEY
);

CREATE TABLE tasks (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN NOT NULL DEFAULT 0,
    deadline_unixtimestamp INTEGER NOT NULL,
    given_by INTEGER NOT NULL
);

CREATE TABLE task_targets (
    task_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (task_id, user_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(discord_id) ON DELETE CASCADE
);

CREATE TABLE reminders (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    task INTEGER NOT NULL,
    when_unixtimestamp INTEGER NOT NULL,
    FOREIGN KEY (task) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE TABLE event_reminders (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    way INTEGER NOT NULL,
    email TEXT,
    FOREIGN KEY (user_id) REFERENCES users(discord_id) ON DELETE CASCADE
);

CREATE INDEX idx_reminders_task ON reminders(task);
CREATE INDEX idx_task_targets_user_id ON task_targets(user_id);

