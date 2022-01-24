mod vincenty;

use tide::prelude::*;
use std::str::FromStr;
use vincenty::{GeoCoordinate, distance};

#[derive(Deserialize, Debug)]
#[serde(default)]
struct Query {
    src: String,
    dst: String
}

impl Default for Query {
    fn default() -> Self {
        Self {
            src: "".to_string(),
            dst: "".to_string()
        }
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/distance").post(|req: tide::Request<()>| async move {
        let q: Query = req.query()?;
        let c1: GeoCoordinate = GeoCoordinate::from_str(&q.src)?;
        let c2: GeoCoordinate = GeoCoordinate::from_str(&q.dst)?;
        Ok(json!({"data": {"src": c1, "dst": c2, "distance": distance(&c1, &c2)}}))
    });

    app.listen("localhost:5000").await?;

    Ok(())

}
