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
//! let polygons: geo::MultiPolygon<f64> = geo::MultiPolygon::rand(&mut rng, &GeoRandParameters::default());
//! # }
//! ```
//!
//! [`GeoRandParameters`] contains fields to customize output.
//!
//! [`GeoRand`]: trait.GeoRand.html
//! [`rand`]: trait.GeoRand.html#method.rand
//! [`GeoRandParameters`]: struct.GeoRandParameters.html

use geo::algorithm::{intersects::Intersects, translate::Translate};
use num_traits::{Float, Num, NumCast};
use rand::distributions::uniform::SampleUniform;
use rand::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GeoRandParameters<T: Copy + PartialOrd<T> + NumCast + Num> {
    pub max_polygons_count: usize,
    pub max_polygon_vertices_count: usize,
    pub max_collisions_count: Option<u32>,
    pub min_x: T,
    pub min_y: T,
    pub max_x: T,
    pub max_y: T,
}

impl<T: Copy + PartialOrd<T> + NumCast + Num> Default for GeoRandParameters<T> {
    fn default() -> Self {
        Self {
            max_polygons_count: 60,
            max_polygon_vertices_count: 7,
            max_collisions_count: Some(100),
            min_x: T::zero(),
            min_y: T::zero(),
            max_x: T::from(100.0).unwrap(),
            max_y: T::from(100.0).unwrap(),
        }
    }
}

pub trait GeoRand<T: Copy + PartialOrd<T> + NumCast + Num> {
    fn rand(rng: &mut impl Rng, geo_rand_parameters: &GeoRandParameters<T>) -> Self;
}

impl<T: Copy + PartialOrd<T> + NumCast + Num + Float + SampleUniform> GeoRand<T>
    for geo::MultiPolygon<T>
{
    fn rand(rng: &mut impl Rng, parameters: &GeoRandParameters<T>) -> Self {
        let mut polygons = Vec::with_capacity(parameters.max_polygons_count);
        let mut collisions_count = 0;

        'outer: while parameters
            .max_collisions_count
            .and_then(|max_collisions_count| Some(collisions_count < max_collisions_count))
            .unwrap_or(true)
            && polygons.len() < parameters.max_polygons_count
        {
            let new_polygon = geo::Polygon::rand(rng, parameters);

            if parameters.max_collisions_count.is_some() {
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

impl<T: Copy + PartialOrd<T> + NumCast + Num + SampleUniform> GeoRand<T> for geo::Polygon<T> {
    fn rand(rng: &mut impl Rng, parameters: &GeoRandParameters<T>) -> Self {
        let bound_x1 = rng.gen_range(parameters.min_x, parameters.max_x);
        let bound_y1 = rng.gen_range(parameters.min_y, parameters.max_y);
        let bound_x2 = rng.gen_range(parameters.min_x, parameters.max_x);
        let bound_y2 = rng.gen_range(parameters.min_y, parameters.max_y);

        let (min_x, max_x) = if bound_x1 < bound_x2 {
            (bound_x1, bound_x2)
        } else {
            (bound_x2, bound_x1)
        };

        let (min_y, max_y) = if bound_y1 < bound_y2 {
            (bound_y1, bound_y2)
        } else {
            (bound_y2, bound_y1)
        };

        let translate_x = rng.gen_range(parameters.min_x - min_x, parameters.max_x - max_x);
        let translate_y = rng.gen_range(parameters.min_y - min_y, parameters.max_y - max_y);
        let vertices_count = rng.gen_range(3, parameters.max_polygon_vertices_count);

        let point_parameters = GeoRandParameters {
            min_x,
            min_y,
            max_x,
            max_y,
            ..*parameters
        };

        let points: Vec<_> = (0..vertices_count)
            .map(|_| geo::Point::rand(rng, &point_parameters))
            .collect();

        geo::Polygon::new(points_to_contour(&points).unwrap(), Vec::new())
            .translate(translate_x, translate_y)
    }
}

impl<T: Copy + PartialOrd<T> + NumCast + Num + SampleUniform> GeoRand<T> for geo::Point<T> {
    fn rand(rng: &mut impl Rng, parameters: &GeoRandParameters<T>) -> Self {
        geo::Point::new(
            rng.gen_range(parameters.min_x, parameters.max_x),
            rng.gen_range(parameters.min_y, parameters.max_y),
        )
    }
}

fn points_to_contour<T: Copy + PartialOrd<T> + NumCast + Num>(
    points: &[geo::Point<T>],
) -> Option<geo::LineString<T>> {
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

    let (mut above_list, mut below_list): (Vec<geo::Point<T>>, Vec<geo::Point<T>>) = points
        .iter()
        .filter(|&&point| point != left_most && point != right_most)
        .partition(|&&point| left_turn_test(&(right_most - left_most), &(point - left_most)));

    above_list.sort_by(|a, b| (a.x() - b.x()).partial_cmp(&T::zero()).unwrap());
    below_list.sort_by(|a, b| (b.x() - a.x()).partial_cmp(&T::zero()).unwrap());

    Some(
        std::iter::once(left_most)
            .chain(above_list)
            .chain(std::iter::once(right_most))
            .chain(below_list)
            .chain(std::iter::once(left_most))
            .collect(),
    )
}

fn left_turn_test<T: Copy + PartialOrd<T> + NumCast + Num>(
    point: &geo::Point<T>,
    other_point: &geo::Point<T>,
) -> bool {
    ((point.x() * other_point.y()) - (point.y() * other_point.x())) >= T::zero()
}
