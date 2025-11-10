-- Migration: Add 'cancelled' status to analysis_sessions table
-- Date: 2025-01-28
-- Description: Updates CHECK constraint to allow 'cancelled' status

PRAGMA foreign_keys=off;

BEGIN TRANSACTION;

-- Tạo bảng mới với CHECK constraint đã update
CREATE TABLE analysis_sessions_new (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'cancelled')),
    error_message TEXT,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
);

-- Copy dữ liệu từ bảng cũ
INSERT INTO analysis_sessions_new SELECT * FROM analysis_sessions;

-- Drop bảng cũ
DROP TABLE analysis_sessions;

-- Rename bảng mới
ALTER TABLE analysis_sessions_new RENAME TO analysis_sessions;

COMMIT;

PRAGMA foreign_keys=on;

