mod vincenty;

use cache_2q::Cache;
use http_types::{convert::json, headers::HeaderValue, Body, StatusCode};
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
use tide::security::{CorsMiddleware, Origin};
use tide::{prelude::*, Response};
use vincenty::{distance, GeoCoordinate};

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(default)]
struct Query {
    src: String,
    dst: String,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            src: "".to_string(),
            dst: "".to_string(),
        }
    }
}

#[derive(Clone)]
struct State {
    cache: Arc<RwLock<Cache<Query, f64>>>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let state = State {
        cache: Arc::new(RwLock::new(Cache::new(1024))),
    };

    let mut app = tide::with_state(state);

    let cors = CorsMiddleware::new()
        .allow_methods("GET".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.at("/distance")
        .with(cors)
        .get(|req: tide::Request<State>| async move {
            let q: Query = req.query()?;
            let c1: GeoCoordinate = GeoCoordinate::from_str(&q.src)?;
            let c2: GeoCoordinate = GeoCoordinate::from_str(&q.dst)?;

            let mut cache = req.state().cache.write().unwrap();

            let json = match cache.get(&q) {
                Some(d) => json!({"data": {"src": c1, "dst": c2, "distance": d}}),
                None => {
                    let distance = distance(&c1, &c2)?;
                    cache.insert(q.clone(), distance);
                    json!({"data": {"src": c1, "dst": c2, "distance": distance}})
                }
            };

            let resp = Response::builder(StatusCode::Ok)
                .content_type("application/json")
                .body(Body::from_json(&json)?)
                .build();

            Ok(resp)
        });

    app.listen("localhost:5000").await?;

    Ok(())
}
