-- TokenForest Database Schema
-- SQLite database for token management

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- Tokens table
CREATE TABLE IF NOT EXISTS tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL UNIQUE,
    supply INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_tokens_symbol ON tokens(symbol);
CREATE INDEX IF NOT EXISTS idx_tokens_created ON tokens(created_at);

-- Insert some sample data (optional)
-- INSERT INTO tokens (name, symbol, supply) VALUES 
--     ('Forest Coin', 'FST', 1000000),
--     ('Tree Token', 'TREE', 500000),
--     ('Green Leaf', 'LEAF', 2000000);
