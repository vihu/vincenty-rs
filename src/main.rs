mod vincenty;

use tide::{prelude::*, Response};
use std::str::FromStr;
use vincenty::{GeoCoordinate, distance};
use http_types::{headers::HeaderValue, StatusCode, Body, convert::json};
use tide::security::{CorsMiddleware, Origin};

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

    let cors = CorsMiddleware::new()
        .allow_methods("GET".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.at("/distance")
        .with(cors)
        .get(|req: tide::Request<()>| async move {
        let q: Query = req.query()?;
        let c1: GeoCoordinate = GeoCoordinate::from_str(&q.src)?;
        let c2: GeoCoordinate = GeoCoordinate::from_str(&q.dst)?;
        let distance = distance(&c1, &c2)?;
        let json = json!({"data": {"src": c1, "dst": c2, "distance": distance}});

        let resp =
            Response::builder(StatusCode::Ok)
            .content_type("application/json")
            .body(Body::from_json(&json)?)
            .build();

        Ok(resp)
    });

    app.listen("localhost:5000").await?;

    Ok(())

}
