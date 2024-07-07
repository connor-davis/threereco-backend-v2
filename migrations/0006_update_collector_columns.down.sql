-- Add down migration script here
ALTER TABLE IF EXISTS collector_profile
DROP COLUMN IF EXISTS bank_account_holder,
DROP COLUMN IF EXISTS bank_account_number,
DROP COLUMN IF EXISTS bank_name;