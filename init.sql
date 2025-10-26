-- Create sample table
CREATE TABLE IF NOT EXISTS sample_data (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    value NUMERIC(10, 2) NOT NULL
);

-- Insert sample data
INSERT INTO sample_data (date, value) VALUES
    ('2025-10-10', 100.50),
    ('2025-10-10', 200.75),
    ('2025-10-11', 150.25),
    ('2025-10-12', 300.00),
    ('2025-10-13', 175.80);

-- Create another sample table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample users
INSERT INTO users (name, email) VALUES
    ('Alice Johnson', 'alice@example.com'),
    ('Bob Smith', 'bob@example.com'),
    ('Charlie Brown', 'charlie@example.com');