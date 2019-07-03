//! This crate contains algorithms to generate random geometric shapes such as polygons.
//!
//! This code is a Rust port of [this JS code](https://github.com/fribbels/fribbels.github.io/blob/master/shortestpath/polyutils.js).
//!
//! # Example
//!
//! The following example shows how to compute a random set of polygons.  
//! The [`rand`] method is provided by the [`GeoRand`] trait which is implemented for some [geo-types](https://docs.rs/geo-types/0.4.3/geo_types/).
//!
//! ```rust
//! # fn main() {
//! use rand_core::SeedableRng;
//! use geo_rand::{GeoRand, GeoRandParameters};
//! let mut rng = rand_pcg::Pcg64::seed_from_u64(0);
//! let polygons = geo::MultiPolygon::rand(&mut rng, &GeoRandParameters::default());
//! # }
//! ```
//!
//! [`GeoRandParameters`] contains fields to customize output.
//!
//! [`GeoRand`]: trait.GeoRand.html
//! [`rand`]: trait.GeoRand.html#method.rand
//! [`GeoRandParameters`]: struct.GeoRandParameters.html

use geo::algorithm::{intersects::Intersects, translate::Translate};
use rand::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GeoRandParameters {
    pub max_polygons_count: usize,
    pub max_polygon_vertices_count: usize,
    pub max_collisions_count: Option<u32>,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Default for GeoRandParameters {
    fn default() -> Self {
        Self {
            max_polygons_count: 60,
            max_polygon_vertices_count: 7,
            max_collisions_count: Some(100),
            min_x: 0.0,
            min_y: 0.0,
            max_x: 400.0,
            max_y: 400.0,
        }
    }
}

pub trait GeoRand {
    fn rand(rng: &mut impl Rng, geo_rand_parameters: &GeoRandParameters) -> Self;
}

impl GeoRand for geo::MultiPolygon<f64> {
    fn rand(rng: &mut impl Rng, parameters: &GeoRandParameters) -> Self {
        let mut polygons = Vec::with_capacity(parameters.max_polygons_count);
        let mut collisions_count = 0;

        'outer: while parameters
            .max_collisions_count
            .and_then(|max_collisions_count| Some(collisions_count < max_collisions_count))
            .unwrap_or(true)
            && polygons.len() < parameters.max_polygons_count
        {
            let new_polygon = geo::Polygon::rand(rng, parameters);

            if let Some(_) = parameters.max_collisions_count {
                for polygon in &polygons {
                    if new_polygon.intersects(polygon) {
                        collisions_count += 1;
                        continue 'outer;
                    }
                }
            }

            polygons.push(new_polygon);
        }

        geo::MultiPolygon(polygons)
    }
}

impl GeoRand for geo::Polygon<f64> {
    fn rand(rng: &mut impl Rng, parameters: &GeoRandParameters) -> Self {
        let min_x = rng.gen_range(parameters.min_x, parameters.max_x);
        let min_y = rng.gen_range(parameters.min_y, parameters.max_y);
        let max_x = rng.gen_range(min_x, parameters.max_x);
        let max_y = rng.gen_range(min_y, parameters.max_y);
        let translate_x = rng.gen_range(0.0, parameters.max_x - max_x);
        let translate_y = rng.gen_range(0.0, parameters.max_y - max_y);
        let vertices_count = rng.gen_range(3, parameters.max_polygon_vertices_count);

        let point_parameters = GeoRandParameters {
            min_x,
            min_y,
            max_x,
            max_y,
            ..parameters.clone()
        };

        let points: Vec<_> = (0..vertices_count)
            .map(|_| geo::Point::rand(rng, &point_parameters))
            .collect();

        geo::Polygon::new(points_to_contour(&points).unwrap(), Vec::new())
            .translate(translate_x, translate_y)
    }
}

impl GeoRand for geo::Point<f64> {
    fn rand(rng: &mut impl Rng, parameters: &GeoRandParameters) -> Self {
        geo::Point::new(
            rng.gen_range(parameters.min_x, parameters.max_x),
            rng.gen_range(parameters.min_y, parameters.max_y),
        )
    }
}

fn points_to_contour(points: &[geo::Point<f64>]) -> Option<geo::LineString<f64>> {
    let first_point = *points.get(0)?;
    let (left_most, right_most) = points.iter().skip(1).fold(
        (first_point, first_point),
        |(left_most, right_most), &point| {
            (
                if point.x() < left_most.x() {
                    point
                } else {
                    left_most
                },
                if point.x() >= right_most.x() {
                    point
                } else {
                    right_most
                },
            )
        },
    );

    let (mut above_list, mut below_list): (Vec<geo::Point<f64>>, Vec<geo::Point<f64>>) = points
        .iter()
        .filter(|&&point| point != left_most && point != right_most)
        .partition(|&&point| left_turn_test(&(right_most - left_most), &(point - left_most)));

    above_list.sort_by(|a, b| (a.x() - b.x()).partial_cmp(&0.0).unwrap());
    below_list.sort_by(|a, b| (b.x() - a.x()).partial_cmp(&0.0).unwrap());

    Some(
        std::iter::once(left_most)
            .chain(above_list)
            .chain(std::iter::once(right_most))
            .chain(below_list)
            .chain(std::iter::once(left_most))
            .collect(),
    )
}

fn left_turn_test(point: &geo::Point<f64>, other_point: &geo::Point<f64>) -> bool {
    ((point.x() * other_point.y()) - (point.y() * other_point.x())).is_sign_positive()
}
