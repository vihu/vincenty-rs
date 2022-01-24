use serde::Serialize;
use std::str::FromStr;

const RADIUS_AT_EQUATOR: f64 = 6_378_137.0;
const FLATTENING_ELIPSOID: f64 = 1.0 / 298.257_223_563;
const RADIUS_AT_POLES: f64 = (1.0 - FLATTENING_ELIPSOID) * RADIUS_AT_EQUATOR;
const MAX_ITERATIONS: u32 = 200;
const CONVERGENCE_THRESHOLD: f64 = 0.000_000_000_001;
const PRECISION: i32 = 6;

#[derive(Debug, Serialize, Clone)]
pub struct GeoCoordinate {
    lat: f64,
    lng: f64,
}

impl GeoCoordinate {
    fn new(lat: f64, lng: f64) -> Self {
        GeoCoordinate { lat, lng }
    }
}

impl FromStr for GeoCoordinate {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (src, dst) = s.split_once(",").expect("Incorrect format!");

        Ok(GeoCoordinate {
            lat: src.trim().parse().unwrap(),
            lng: dst.trim().parse().unwrap(),
        })
    }
}

pub fn distance(c1: &GeoCoordinate, c2: &GeoCoordinate) -> Option<f64> {
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
) -> Option<f64> {
    for _ in 0..MAX_ITERATIONS {
        let sin_lambda = f64::sin(lambda);
        let cos_lambda = f64::cos(lambda);
        let sin_sigma = f64::sqrt(
            f64::powi(cos_u2 * sin_lambda, 2)
                + f64::powi(cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda, 2),
        );

        if sin_sigma == 0.0 {
            return Some(0.0);
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
            return Some(round(
                evaluate(cos_sqalpha, sin_sigma, cos2_sigma_m, cos_sigma, sigma),
                PRECISION,
            ));
        }

        lambda = new_lambda;
    }

    None
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
            distance(&GeoCoordinate::new(0.0, 0.0), &GeoCoordinate::new(0.0, 0.0)),
            Some(0.0)
        );
    }

    #[test]
    fn basic() {
        assert_eq!(
            distance(
                &GeoCoordinate::new(42.3541165, -71.0693514),
                &GeoCoordinate::new(40.7791472, -73.9680804)
            ),
            Some(298.396186)
        )
    }

    #[test]
    fn known() {
        assert_eq!(
            distance(
                &GeoCoordinate::new(39.152501, -84.412977),
                &GeoCoordinate::new(39.152505, -84.412946)
            ),
            Some(0.002716)
        )
    }
}
