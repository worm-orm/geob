use geo::{Coord, coord, point};
use geob::Geob;
use rusqlite::Connection;

fn main() -> rusqlite::Result<()> {
    let db = Connection::open_in_memory()?;

    rusqlite_geob::register(&db)?;

    // let geo = geo::Rect::new(
    //     coord! {x: 12.559471, y: 55.673728},
    //     coord!(x: 12.560254, y: 55.674158),
    // );

    let loppen = point!(x: 12.597135, y: 55.673891);
    let rust = point!(x: 12.559285, y: 55.691249);
    let lygten = point!(x:12.5378308, y: 55.7036352);

    let loppen = Geob::from_geo_type(&loppen, 3857);
    let rust = Geob::from_geo_type(&rust, 3857);
    let lygten = Geob::from_geo_type(&lygten, 3857);

    db.execute("CREATE TABLE test(point blob, name text)", ())?;

    db.execute("CREATE VIRTUAL TABLE test_index USING SpartialIndex(table = 'test', column = 'point', srid = 3857, type = 'point')", [])?;

    db.execute("INSERT INTO test VALUES (?, ?)", (&loppen, "Loppen"))?;
    db.execute("INSERT INTO test VALUES (?, ?)", (&rust, "Rust"))?;
    db.execute("INSERT INTO test VALUES (?, ?)", (&lygten, "Lygten"))?;

    let mut stmt =
        db.prepare("SELECT name, point, St_Distance(point, $1) / 1000 FROM test WHERE rowid in (SELECT id FROM test_index where distance < 3000 and geometry = $1) and name is not 'Lygten'")?;

    let mut rows = stmt.query((&lygten,))?;

    while let Some(next) = rows.next()? {
        let name: String = next.get(0)?;
        let geo: Geob = next.get(1)?;
        let dis: f64 = next.get(2)?;

        println!("ID {}, GEOM {}: {}", name, geo, dis);
    }

    db.execute("DELETE FROM test where rowid = 2", [])?;

    let mut stmt = db.prepare("SELECT id, ST_ToText(geometry) FROM test_index")?;

    let mut rows = stmt.query(())?;

    while let Some(next) = rows.next()? {
        let name: u64 = next.get(0)?;
        let geo: Geob = next.get(1)?;
        // let dis: f64 = next.get(2)?;

        println!("ID {}, GEOM {}", name, geo);
    }

    // db.query_one("SELECT ST_AddColumn('test', 'point', 3857)", [], |_| Ok(()))?;

    // db.query_one("SELECT ST_CreateIndex('test', 'point')", [], |row| {
    //     row.get::<_, bool>(0)
    // })?;

    // let proj = Geob::from_geo_type(&geo, 4326).project_into(3857);

    // println!("{proj}");

    // db.execute("INSERT INTO test (point) VALUES (?)", (proj,))?;

    // let id: String = db.query_one(
    //     "SELECT ST_ToText(point) FROM test WHERE rowid in (SELECT id FROM SpartialIndex WHERE distance = 10 AND geometry match 20)",
    //     [],
    //     |ctx| ctx.get(0),
    // )?;

    // println!("ID {}", id);

    // db.execute("CREATE VIRTUAL TABLE SpartialIndex USING SpartialIndex", [])?;

    // let out: Geob = db.query_one(
    //     "SELECT point, (ST_Distance(point, ST_FromText(?)) / 1000) FROM test",
    //     ("SRID=4326;POINT(13.212 34.2030)",),
    //     |row| row.get(0),
    // )?;

    // println!("{}", out);

    // db.execute(
    //     "INSERT INTO SpartialIndex (id, table_name, geometry) VALUES(1, 'test', ?)",
    //     [Geob::from_geo_type(&geo, 4326)],
    // )?;

    // db.execute(
    //     "UPDATE SpartialIndex SET geometry = ?",
    //     [Geob::from_geo_type(&geo, 4326)],
    // )?;

    // let out: Option<f64> = db.query_one(
    //     "SELECT ST_Area(ST_Transform(point, 4326), false) FROM test",
    //     [],
    //     |row| row.get(0),
    // )?;

    // println!("{:?}", out);

    Ok(())
}
