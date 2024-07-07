-- Add down migration script here
ALTER TABLE IF EXISTS product
ALTER COLUMN price TYPE DECIMAL;

ALTER TABLE IF EXISTS collection
ALTER COLUMN weight TYPE DECIMAL;