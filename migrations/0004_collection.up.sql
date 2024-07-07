-- Add up migration script here
CREATE TABLE
    IF NOT EXISTS product (
        id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (),
        business_id UUID NOT NULL,
        name VARCHAR(255) NOT NULL,
        description TEXT NOT NULL,
        price DECIMAL NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (business_id) REFERENCES business_profile (id)
    );

CREATE TABLE
    IF NOT EXISTS collection (
        id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (),
        business_id UUID NOT NULL,
        collector_id UUID NOT NULL,
        product_id UUID NOT NULL,
        weight DECIMAL NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (business_id) REFERENCES business_profile (id),
        FOREIGN KEY (collector_id) REFERENCES collector_profile (id),
        FOREIGN KEY (product_id) REFERENCES product (id)
    );