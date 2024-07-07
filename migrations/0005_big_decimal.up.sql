-- Add up migration script here
-- update the price column to be a DECIMAL(10, 2) type
ALTER TABLE IF EXISTS product
ALTER COLUMN price TYPE DECIMAL(10, 2);

-- update the weight column to be a DECIMAL(10, 2) type
ALTER TABLE IF EXISTS collection
ALTER COLUMN weight TYPE DECIMAL(10, 2);