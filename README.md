# geo-rand

This crate contains algorithms to generate random geometric shapes such as polygons.

This code is a Rust port of [this JS code](https://github.com/fribbels/fribbels.github.io/blob/master/shortestpath/polyutils.js).

## Example

The following example shows how to compute a random set of polygons.
The [`rand`] method is provided by the [`GeoRand`] trait which is implemented for some [geo-types](https://docs.rs/geo-types/0.4.3/geo_types/).

```rust
use rand_core::SeedableRng;
use geo_rand::{GeoRand, GeoRandParameters};
let mut rng = rand_pcg::Pcg64::seed_from_u64(0);
let polygons = geo::MultiPolygon::rand(&mut rng, &GeoRandParameters::default());
```

[`GeoRandParameters`] contains fields to customize output.

[`GeoRand`]: trait.GeoRand.html
[`rand`]: trait.GeoRand.html#method.rand
[`GeoRandParameters`]: struct.GeoRandParameters.html
