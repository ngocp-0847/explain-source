-- Migration 003: Add authentication and plan collaboration features

-- Create users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Add mode field to tickets table
ALTER TABLE tickets ADD COLUMN mode TEXT CHECK(mode IN ('plan', 'ask', 'edit')) DEFAULT 'ask';

-- Add plan collaboration fields to tickets table
ALTER TABLE tickets ADD COLUMN plan_content TEXT;
ALTER TABLE tickets ADD COLUMN plan_created_at TEXT;
ALTER TABLE tickets ADD COLUMN required_approvals INTEGER DEFAULT 2;

-- Create plan_edits table for tracking edit history
CREATE TABLE IF NOT EXISTS plan_edits (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    content_before TEXT,
    content_after TEXT,
    edited_at TEXT NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create plan_approvals table for approval workflow
CREATE TABLE IF NOT EXISTS plan_approvals (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    approved_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('approved', 'rejected')),
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(ticket_id, user_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_plan_edits_ticket_id ON plan_edits(ticket_id);
CREATE INDEX IF NOT EXISTS idx_plan_approvals_ticket_id ON plan_approvals(ticket_id);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

