CREATE OR REPLACE FUNCTION calculate_karma()
RETURNS TRIGGER AS $$
BEGIN
    NEW.karma := ROW(
        0,          -- amount
        0,    -- reviews
        EXTRACT(YEAR FROM AGE(NOW(), NEW.created_at)), -- age
        0         -- popularity
    )::karma;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_karma
BEFORE INSERT OR UPDATE ON subject.website
FOR EACH ROW
EXECUTE FUNCTION calculate_karma();