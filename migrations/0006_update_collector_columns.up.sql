-- Add up migration script here
ALTER TABLE IF EXISTS collector_profile
ADD COLUMN IF NOT EXISTS bank_account_holder VARCHAR(255) NOT NULL,
ADD COLUMN IF NOT EXISTS bank_account_number VARCHAR(255) NOT NULL,
ADD COLUMN IF NOT EXISTS bank_name VARCHAR(255) NOT NULL;