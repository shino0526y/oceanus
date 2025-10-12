CREATE TRIGGER trg_bu_application_entities_set_updated_at
    BEFORE UPDATE ON application_entities
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

