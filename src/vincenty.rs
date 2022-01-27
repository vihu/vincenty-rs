use anyhow::{bail, Result};
use h3ron::{FromH3Index, H3Cell, ToCoordinate};
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

const RADIUS_AT_EQUATOR: f64 = 6_378_137.0;
const FLATTENING_ELIPSOID: f64 = 1.0 / 298.257_223_563;
const RADIUS_AT_POLES: f64 = (1.0 - FLATTENING_ELIPSOID) * RADIUS_AT_EQUATOR;
const MAX_ITERATIONS: u32 = 200;
const CONVERGENCE_THRESHOLD: f64 = 0.000_000_000_001;
const PRECISION: i32 = 6;

#[derive(Debug, Clone, Serialize)]
pub struct GeoCoordinate {
    lat: f64,
    lng: f64,
}

#[allow(dead_code)]
impl GeoCoordinate {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

impl Display for GeoCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.lat, self.lng)
    }
}

impl TryFrom<u64> for GeoCoordinate {
    type Error = &'static str;

    fn try_from(index: u64) -> Result<Self, Self::Error> {
        let c = H3Cell::from_h3index(index).to_coordinate();
        Ok(Self { lat: c.y, lng: c.x })
    }
}

impl TryFrom<H3Cell> for GeoCoordinate {
    type Error = &'static str;

    fn try_from(cell: H3Cell) -> Result<Self, Self::Error> {
        let c = cell.to_coordinate();
        Ok(Self { lat: c.y, lng: c.x })
    }
}

impl FromStr for GeoCoordinate {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match H3Cell::from_str(s) {
            Ok(cell) => Ok(GeoCoordinate::try_from(cell).unwrap()),
            Err(_) => match s.parse::<u64>() {
                Ok(h3index) => Ok(GeoCoordinate::try_from(h3index).unwrap()),
                Err(_) => {
                    let (src, dst) = s.trim().split_once(",").expect("Incorrect format!");

                    Ok(GeoCoordinate {
                        lat: src.trim().parse().unwrap(),
                        lng: dst.trim().parse().unwrap(),
                    })
                }
            },
        }
    }
}

pub fn distance(c1: &GeoCoordinate, c2: &GeoCoordinate) -> Result<f64> {
    let u1 = f64::atan((1.0 - FLATTENING_ELIPSOID) * f64::tan(f64::to_radians(c1.lat)));
    let u2 = f64::atan((1.0 - FLATTENING_ELIPSOID) * f64::tan(f64::to_radians(c2.lat)));
    let init_lambda = f64::to_radians(c2.lng - c1.lng);
    let lambda = init_lambda;
    let sin_u1 = f64::sin(u1);
    let cos_u1 = f64::cos(u1);
    let sin_u2 = f64::sin(u2);
    let cos_u2 = f64::cos(u2);

    // approximate till ?MAX_ITERATIONS
    approximate(init_lambda, lambda, sin_u1, cos_u1, sin_u2, cos_u2)
}

fn approximate(
    init_lambda: f64,
    mut lambda: f64,
    sin_u1: f64,
    cos_u1: f64,
    sin_u2: f64,
    cos_u2: f64,
) -> Result<f64> {
    for _ in 0..MAX_ITERATIONS {
        let sin_lambda = f64::sin(lambda);
        let cos_lambda = f64::cos(lambda);
        let sin_sigma = f64::sqrt(
            f64::powi(cos_u2 * sin_lambda, 2)
                + f64::powi(cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda, 2),
        );

        if sin_sigma == 0.0 {
            return Ok(0.0);
        }

        let cos_sigma = sin_u1.mul_add(sin_u2, cos_u1 * cos_u2 * cos_lambda);

        let sigma = f64::atan2(sin_sigma, cos_sigma);
        let sin_alpha = cos_u1 * cos_u2 * sin_lambda / sin_sigma;
        let cos_sqalpha = 1.0 - f64::powi(sin_alpha, 2);

        let cos2_sigma_m = if cos_sqalpha == 0.0 {
            0.0
        } else {
            cos_sigma - 2.0 * sin_u1 * sin_u2 / cos_sqalpha
        };

        let c = (FLATTENING_ELIPSOID / 16.0)
            * cos_sqalpha
            * (4.0 + FLATTENING_ELIPSOID - 3.0 * cos_sqalpha);

        let new_lambda = ((1.0 - c) * FLATTENING_ELIPSOID * sin_alpha).mul_add(
            (c * sin_sigma).mul_add(
                (c * cos_sigma).mul_add(
                    2.0_f64.mul_add(f64::powi(cos2_sigma_m, 2), -1.0),
                    cos2_sigma_m,
                ),
                sigma,
            ),
            init_lambda,
        );

        if f64::abs(new_lambda - lambda) < CONVERGENCE_THRESHOLD {
            // successful
            return Ok(round(
                evaluate(cos_sqalpha, sin_sigma, cos2_sigma_m, cos_sigma, sigma),
                PRECISION,
            ));
        }

        lambda = new_lambda;
    }

    bail!("boom")
}

fn evaluate(
    cos_sqalpha: f64,
    sin_sigma: f64,
    cos2_sigma_m: f64,
    cos_sigma: f64,
    sigma: f64,
) -> f64 {
    let usq = cos_sqalpha * (f64::powi(RADIUS_AT_EQUATOR, 2) - f64::powi(RADIUS_AT_POLES, 2))
        / f64::powi(RADIUS_AT_POLES, 2);
    let a = (usq / 16384.0).mul_add(
        usq.mul_add(usq.mul_add(320.0 - 175.0 * usq, -768.0), 4096.0),
        1.0,
    );
    let b = (usq / 1024.0) * usq.mul_add(usq.mul_add(74.0 - 47.0 * usq, -128.0), 256.0);
    let delta_sigma = b
        * sin_sigma
        * (b / 4.0).mul_add(
            cos_sigma * 2.0_f64.mul_add(f64::powi(cos2_sigma_m, 2), -1.0)
                - (b / 6.0)
                    * cos2_sigma_m
                    * (4.0_f64.mul_add(f64::powi(sin_sigma, 2), -3.0))
                    * (4.0_f64.mul_add(f64::powi(cos2_sigma_m, 2), -3.0)),
            cos2_sigma_m,
        );
    RADIUS_AT_POLES * a * (sigma - delta_sigma) / 1000.0
}

fn round(number: f64, precision: i32) -> f64 {
    let p = f64::powi(10.0, precision);
    f64::round(number * p) / p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        assert_eq!(
            distance(&GeoCoordinate::new(0.0, 0.0), &GeoCoordinate::new(0.0, 0.0)).unwrap(),
            0.0
        );
    }

    #[test]
    fn basic() {
        assert_eq!(
            distance(
                &GeoCoordinate::new(42.3541165, -71.0693514),
                &GeoCoordinate::new(40.7791472, -73.9680804)
            )
            .unwrap(),
            298.396186
        )
    }

    #[test]
    fn known() {
        assert_eq!(
            distance(
                &GeoCoordinate::new(39.152501, -84.412977),
                &GeoCoordinate::new(39.152505, -84.412946)
            )
            .unwrap(),
            0.002716
        )
    }

    #[test]
    fn h3() {
        let c1 = GeoCoordinate::from_str("8826085a4dfffff").unwrap();
        let c2 = GeoCoordinate::from_str("8826085a4dfffff").unwrap();
        assert_eq!(0.0, distance(&c1, &c2).unwrap())
    }
}
