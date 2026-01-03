CREATE TRIGGER trg_bu_application_entities_set_updated_at
    BEFORE UPDATE ON application_entities
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_bu_patients_set_updated_at
    BEFORE UPDATE ON patients
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_bu_studies_set_updated_at
    BEFORE UPDATE ON studies
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_bu_series_set_updated_at
    BEFORE UPDATE ON series
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_bu_sop_instances_set_updated_at
    BEFORE UPDATE ON sop_instances
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

