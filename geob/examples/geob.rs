use geo::{Distance, Euclidean, Geodesic, Haversine, Rhumb};
use geo_traits::to_geo::ToGeoGeometry;
use geo_types::point;
use geob::{Geob, SRID};
fn main() -> Result<(), Box<dyn core::error::Error>> {
    let geob = Geob::from_text("SRID=2020;POLYGON((12.00012 54.0000, 11.0000 203.000))")?;

    let rust = point!(x: 12.559285, y: 55.691249);
    let lygten = point!(x:12.5378308, y: 55.7036352);
    // println!("{:?}", geob);

    let rust = Geob::from_geo_type(&rust, SRID::WEB_MERCATOR.into());
    let lygten = Geob::from_geo_type(&lygten, SRID::WEB_MERCATOR.into());

    println!(
        "Distance {}",
        Haversine.distance(
            rust.geometry().to_geometry().into_point().unwrap(),
            lygten.geometry().to_geometry().into_point().unwrap()
        )
    );

    println!(
        "Distance {}",
        Haversine.distance(
            rust.project_into(4096)
                .geometry()
                .to_geometry()
                .into_point()
                .unwrap(),
            lygten
                .project_into(4096)
                .geometry()
                .to_geometry()
                .into_point()
                .unwrap()
        )
    );

    println!("{:?}", geob.kind());

    Ok(())
}
