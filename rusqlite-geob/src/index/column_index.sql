CREATE TRIGGER IF NOT EXISTS ${name}_insert
  AFTER INSERT ON ${table}
BEGIN
  INSERT INTO ${index} (id, geometry) VALUES(new.rowid, new.${column});
END;
CREATE TRIGGER IF NOT EXISTS ${name}_delete
  AFTER DELETE ON ${table}
BEGIN
  DELETE FROM ${index} WHERE geometry = old.${column} and id = old.rowid;
END;
CREATE TRIGGER IF NOT EXISTS ${name}_update
  AFTER UPDATE ON ${table}
BEGIN
  DELETE FROM ${index} WHERE geometry = old.${column} and id = old.rowid;
  INSERT INTO ${index} (id, geometry) VALUES(new.rowid, new.${column});
END;