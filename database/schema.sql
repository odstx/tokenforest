-- TokenForest Database Schema
-- SQLite database for token management

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
