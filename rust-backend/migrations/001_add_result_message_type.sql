-- Migration: Add 'result' message type to structured_logs table
-- Date: 2025-01-28
-- Description: Updates CHECK constraint to allow 'result' message type

PRAGMA foreign_keys=off;

BEGIN TRANSACTION;

-- Tạo bảng mới với CHECK constraint đã update
CREATE TABLE structured_logs_new (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL,
    message_type TEXT NOT NULL CHECK(message_type IN ('tool_use', 'assistant', 'error', 'system', 'result')),
    content TEXT NOT NULL,
    raw_log TEXT,
    metadata TEXT,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
);

-- Copy dữ liệu từ bảng cũ
INSERT INTO structured_logs_new SELECT * FROM structured_logs;

-- Drop bảng cũ
DROP TABLE structured_logs;

-- Rename bảng mới
ALTER TABLE structured_logs_new RENAME TO structured_logs;

-- Tạo lại indexes
CREATE INDEX idx_logs_ticket_id ON structured_logs(ticket_id);
CREATE INDEX idx_logs_timestamp ON structured_logs(timestamp);

COMMIT;

PRAGMA foreign_keys=on;
