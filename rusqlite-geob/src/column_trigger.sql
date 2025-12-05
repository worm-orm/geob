CREATE TRIGGER IF NOT EXISTS ${name}_insert
  BEFORE INSERT ON ${table}
BEGIN
  SELECT
    CASE 
      WHEN ST_GetSRID(new.${column}) <> ${srid} THEN
        RAISE(ABORT, 'Invalid SRID')
      END;
END;
CREATE TRIGGER IF NOT EXISTS ${name}_update
  BEFORE UPDATE OF ${column} ON ${table}
BEGIN
  SELECT
    CASE 
      WHEN ST_GetSRID(new.${column}) <> ${srid} THEN
        RAISE(ABORT, 'Invalid SRID')
      END;
END;

