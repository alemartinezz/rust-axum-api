-- Database functions and utilities
-- These functions are created in each schema as needed

-- Function to automatically update the updated_at column
-- This function is used by triggers to maintain timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';