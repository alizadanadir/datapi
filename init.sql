-- Drop existing tables if they exist
DROP TABLE IF EXISTS loans CASCADE;
DROP TABLE IF EXISTS customers CASCADE;

-- Create customers table
CREATE TABLE customers (
    customer_id SERIAL PRIMARY KEY,
    country VARCHAR(100) NOT NULL,
    birth_date DATE NOT NULL,
    full_name VARCHAR(200) NOT NULL,
    email VARCHAR(150) NOT NULL,
    phone VARCHAR(20),
    registration_date DATE DEFAULT CURRENT_DATE,
    status VARCHAR(20) DEFAULT 'active'
);

-- Create loans table
CREATE TABLE loans (
    loan_id SERIAL PRIMARY KEY,
    contract_number VARCHAR(50) NOT NULL,
    report_date DATE NOT NULL,
    debt_amount NUMERIC(12, 2) NOT NULL,
    overdue_debt_amount NUMERIC(12, 2) DEFAULT 0.00,
    customer_id INTEGER NOT NULL,
    interest_rate NUMERIC(5, 2) NOT NULL,
    loan_type VARCHAR(50) NOT NULL,
    loan_status VARCHAR(20) DEFAULT 'active',
    disbursement_date DATE NOT NULL,
    maturity_date DATE NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id) ON DELETE CASCADE,
    UNIQUE (contract_number, report_date)
);

-- Create indexes for better query performance
CREATE INDEX idx_customers_country ON customers(country);
CREATE INDEX idx_customers_birth_date ON customers(birth_date);
CREATE INDEX idx_loans_customer_id ON loans(customer_id);
CREATE INDEX idx_loans_report_date ON loans(report_date);
CREATE INDEX idx_loans_status ON loans(loan_status);

-- Insert 1000+ customers from various countries
INSERT INTO customers (country, birth_date, full_name, email, phone, registration_date, status)
SELECT
    CASE (random() * 9)::int
        WHEN 0 THEN 'USA'
        WHEN 1 THEN 'UK'
        WHEN 2 THEN 'Germany'
        WHEN 3 THEN 'France'
        WHEN 4 THEN 'Spain'
        WHEN 5 THEN 'Italy'
        WHEN 6 THEN 'Canada'
        WHEN 7 THEN 'Australia'
        WHEN 8 THEN 'Japan'
        ELSE 'Brazil'
    END,
    DATE '1950-01-01' + (random() * 25000)::int,
    (CASE (random() * 19)::int
        WHEN 0 THEN 'John Smith'
        WHEN 1 THEN 'Jane Johnson'
        WHEN 2 THEN 'Michael Williams'
        WHEN 3 THEN 'Sarah Brown'
        WHEN 4 THEN 'David Jones'
        WHEN 5 THEN 'Emily Garcia'
        WHEN 6 THEN 'Robert Miller'
        WHEN 7 THEN 'Lisa Davis'
        WHEN 8 THEN 'James Rodriguez'
        WHEN 9 THEN 'Maria Martinez'
        WHEN 10 THEN 'William Anderson'
        WHEN 11 THEN 'Anna Taylor'
        WHEN 12 THEN 'Richard Thomas'
        WHEN 13 THEN 'Emma Moore'
        WHEN 14 THEN 'Thomas Jackson'
        WHEN 15 THEN 'Olivia Martin'
        WHEN 16 THEN 'Charles Lee'
        WHEN 17 THEN 'Sophie White'
        WHEN 18 THEN 'Daniel Harris'
        ELSE 'Isabella Clark'
    END),
    'customer' || generate_series || '@email.com',
    '+1-' || LPAD((random() * 9999999999)::bigint::text, 10, '0'),
    DATE '2020-01-01' + (random() * 1800)::int,
    CASE (random() * 3)::int
        WHEN 0 THEN 'active'
        WHEN 1 THEN 'active'
        WHEN 2 THEN 'active'
        ELSE 'inactive'
    END
FROM generate_series(1, 1200) AS generate_series;

-- Insert loans with multiple report dates per contract (showing debt progression)
-- First, create base loans (300 unique contracts)
WITH base_contracts AS (
    SELECT
        generate_series as contract_seq,
        'LOAN-' || LPAD(generate_series::text, 8, '0') as contract_number,
        (random() * 1199 + 1)::int as customer_id,
        (random() * 10 + 3)::numeric(5, 2) as interest_rate,
        CASE (random() * 4)::int
            WHEN 0 THEN 'Personal Loan'
            WHEN 1 THEN 'Mortgage'
            WHEN 2 THEN 'Auto Loan'
            WHEN 3 THEN 'Business Loan'
            ELSE 'Student Loan'
        END as loan_type,
        DATE '2023-01-01' + (random() * 365)::int as disbursement_date,
        DATE '2026-01-01' + (random() * 1095)::int as maturity_date,
        (random() * 40000 + 5000)::numeric(12, 2) as initial_amount
    FROM generate_series(1, 300)
),
report_dates AS (
    SELECT 
        bc.*,
        days.month_offset,
        bc.disbursement_date + (days.month_offset * 30) as report_date
    FROM base_contracts bc
    CROSS JOIN (
        SELECT generate_series as month_offset 
        FROM generate_series(0, 8)
    ) as days
    WHERE bc.disbursement_date + (days.month_offset * 30) <= CURRENT_DATE
)
INSERT INTO loans (
    contract_number,
    report_date,
    debt_amount,
    overdue_debt_amount,
    customer_id,
    interest_rate,
    loan_type,
    loan_status,
    disbursement_date,
    maturity_date
)
SELECT
    rd.contract_number,
    rd.report_date,
    -- Debt DECREASES over time as customer pays
    -- Some loans are paid regularly (decreasing), some have problems (increasing overdue)
    CASE 
        -- Good customers: debt decreases steadily
        WHEN rd.contract_seq % 3 != 0 THEN
            GREATEST(0, rd.initial_amount * (1 - rd.month_offset * 0.11))::numeric(12, 2)
        -- Problem customers: debt decreases slowly or stays high
        ELSE
            GREATEST(0, rd.initial_amount * (1 - rd.month_offset * 0.05))::numeric(12, 2)
    END as debt_amount,
    -- Overdue amount increases for problem loans
    CASE 
        WHEN rd.month_offset > 3 AND (rd.contract_seq % 3 = 0)
        THEN LEAST(rd.initial_amount * 0.3, (rd.initial_amount * 0.05 * (rd.month_offset - 3)))::numeric(12, 2)
        WHEN rd.month_offset > 6 AND (rd.contract_seq % 4 = 0)
        THEN LEAST(rd.initial_amount * 0.5, (rd.initial_amount * 0.08 * (rd.month_offset - 6)))::numeric(12, 2)
        ELSE 0.00
    END as overdue_debt_amount,
    rd.customer_id,
    rd.interest_rate,
    rd.loan_type,
    CASE 
        WHEN rd.month_offset > 8 AND (rd.contract_seq % 10 = 0) THEN 'defaulted'
        WHEN rd.month_offset > 5 AND (rd.contract_seq % 5 = 0) THEN 'overdue'
        WHEN rd.initial_amount * (1 - rd.month_offset * 0.11) <= 0 THEN 'paid_off'
        ELSE 'active'
    END as loan_status,
    rd.disbursement_date,
    rd.maturity_date
FROM report_dates rd;

-- Add some additional loans for existing customers to show relationships
INSERT INTO loans (
    contract_number,
    report_date,
    debt_amount,
    overdue_debt_amount,
    customer_id,
    interest_rate,
    loan_type,
    loan_status,
    disbursement_date,
    maturity_date
)
SELECT
    'LOAN-EXTRA-' || LPAD(generate_series::text, 6, '0'),
    DATE '2024-01-01' + (generate_series % 300),
    (random() * 30000 + 500)::numeric(12, 2),
    CASE 
        WHEN random() < 0.4 THEN (random() * 8000)::numeric(12, 2)
        ELSE 0.00
    END,
    (random() * 100 + 1)::int,
    (random() * 8 + 4)::numeric(5, 2),
    CASE (random() * 2)::int
        WHEN 0 THEN 'Credit Card'
        WHEN 1 THEN 'Line of Credit'
        ELSE 'Payday Loan'
    END,
    CASE (random() * 3)::int
        WHEN 0 THEN 'active'
        WHEN 1 THEN 'active'
        ELSE 'overdue'
    END,
    DATE '2023-06-01' + (random() * 500)::int,
    DATE '2025-01-01' + (random() * 1825)::int
FROM generate_series(1, 300) AS generate_series;

-- Create a view for easy loan summary by customer
CREATE OR REPLACE VIEW customer_loan_summary AS
SELECT
    c.customer_id,
    c.country,
    c.full_name,
    COUNT(l.loan_id) as total_loans,
    SUM(l.debt_amount) as total_debt,
    SUM(l.overdue_debt_amount) as total_overdue,
    MAX(l.report_date) as latest_report_date
FROM customers c
LEFT JOIN loans l ON c.customer_id = l.customer_id
GROUP BY c.customer_id, c.country, c.full_name;

-- Display summary statistics
SELECT 'Customers created' as metric, COUNT(*)::text as value FROM customers
UNION ALL
SELECT 'Loans created' as metric, COUNT(*)::text as value FROM loans
UNION ALL
SELECT 'Countries represented' as metric, COUNT(DISTINCT country)::text as value FROM customers
UNION ALL
SELECT 'Active loans' as metric, COUNT(*)::text as value FROM loans WHERE loan_status = 'active'
UNION ALL
SELECT 'Overdue loans' as metric, COUNT(*)::text as value FROM loans WHERE overdue_debt_amount > 0;