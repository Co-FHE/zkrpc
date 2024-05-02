//! Fast geodesic distance approximations via flat surface projection
//!
//! The [`FlatProjection`] struct can by used to project geographical
//! coordinates from [WGS84] into a cartesian coordinate system.
//! In the projected form approximated distance and bearing calculations
//! can be performed much faster than on a sphere. The precision of these
//! calculations is very precise for distances up to about 500 km.
//!
//! [`FlatProjection`]: struct.FlatProjection.html
//! [WGS84]: https://en.wikipedia.org/wiki/World_Geodetic_System
//!
//! ## Example
//!
//! ```
//! # #[macro_use]
//! # extern crate assert_approx_eq;
//! # extern crate num_traits;
//! # extern crate flat_projection;
//! #
//! # use num_traits::float::Float;
//! # use flat_projection::FlatProjection;
//! #
//! # fn main() {
//! let (lon1, lat1) = (6.186389, 50.823194);
//! let (lon2, lat2) = (6.953333, 51.301389);
//!
//! let proj = FlatProjection::new(6.5, 51.05);
//!
//! let p1 = proj.project(lon1, lat1);
//! let p2 = proj.project(lon2, lat2);
//!
//! let distance = p1.distance(&p2);
//! // -> 75.648 km
//! #
//! # assert_approx_eq!(distance, 75.635_595, 0.02);
//! # }
//! ```
//!
//! ## Related
//!
//! - [cheap-ruler] – Similar calculations in JavaScripts
//! - [cheap-ruler-cpp] – C++ port of cheap-ruler
//!
//! [cheap-ruler]: https://github.com/mapbox/cheap-ruler
//! [cheap-ruler-cpp]: https://github.com/mapbox/cheap-ruler-cpp

#![allow(clippy::derive_partial_eq_without_eq)]

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;
use types::FixedPoint;
use types::Pos2D;
use types::Pos3D;

const RAD: Decimal = dec!(0.0174532925199432957692369077);
const EARTH_RADIUS: Decimal = dec!(6378.137);
// 1/ 298.257223563
// const FLATTENING: Decimal = dec!(0.0033528106647474807198455286);
const E2: Decimal = dec!(0.0066943799901413169961372335);

/// Projection from [WGS84] to a cartesian coordinate system for fast
/// geodesic approximations.
///
/// [WGS84]: https://en.wikipedia.org/wiki/World_Geodetic_System
///
/// ```
/// # #[macro_use]
/// # extern crate assert_approx_eq;
/// # extern crate num_traits;
/// # extern crate flat_projection;
/// #
/// # use num_traits::float::Float;
/// # use flat_projection::FlatProjection;
/// #
/// # fn main() {
/// let (lon1, lat1) = (6.186389, 50.823194);
/// let (lon2, lat2) = (6.953333, 51.301389);
///
/// let proj = FlatProjection::new(6.5, 51.05);
///
/// let p1 = proj.project(lon1, lat1);
/// let p2 = proj.project(lon2, lat2);
///
/// let distance = p1.distance(&p2);
/// // -> 75.648 km
/// #
/// # assert_approx_eq!(distance, 75.635_595, 0.02);
/// # }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct FlatProjection<T: FixedPoint> {
    kx: T,
    ky: T,

    lat: T,
    lon: T,
}

trait ToRadians {
    fn to_radians(&self) -> Self;
}
impl ToRadians for Decimal {
    fn to_radians(&self) -> Self {
        self * RAD
    }
}

impl FlatProjection<Decimal> {
    /// Creates a new `FlatProjection` instance that will work best around
    /// the given latitude.
    ///
    /// ```
    /// # use flat_projection::FlatProjection;
    /// #
    /// let proj = FlatProjection::new(7., 51.);
    /// ```
    pub fn new(longitude: Decimal, latitude: Decimal) -> FlatProjection<Decimal> {
        // see https://github.com/mapbox/cheap-ruler/

        let one = Decimal::ONE;
        // let two = Decimal::TWO;

        // Values that define WGS84 ellipsoid model of the Earth

        // let re: T = T::from(6378.137).unwrap(); // equatorial radius
        // let fe: T = one / T::from(298.257223563).unwrap(); // flattening
        // let e2: T = fe * (two - fe);
        let re: Decimal = EARTH_RADIUS; // equatorial radius
                                        // let fe: Decimal = FLATTENING; // flattening
        let e2: Decimal = E2;
        // let re = dec!(6378.137);
        // let fe = one / dec!(298.257223563);
        // let e2 = fe * (two - fe);

        // Curvature formulas from https://en.wikipedia.org/wiki/Earth_radius#Meridional
        let cos_lat = latitude.to_radians().cos();
        let w2 = one / (one - e2 * (one - cos_lat * cos_lat));
        let w = w2.sqrt().unwrap();

        // multipliers for converting longitude and latitude degrees into distance
        let kx = (re * w * cos_lat).to_radians(); // based on normal radius of curvature
        let ky = (re * w * w2 * (one - e2)).to_radians(); // based on meridional radius of curvature

        FlatProjection {
            kx,
            ky,
            lat: latitude,
            lon: longitude,
        }
    }

    /// Converts a longitude and latitude (in degrees) to a [`FlatPoint`]
    /// instance that can be used for fast geodesic approximations.
    ///
    /// [`FlatPoint`]: struct.FlatPoint.html
    ///
    /// ```
    /// # use flat_projection::FlatProjection;
    /// #
    /// let (lon, lat) = (6.186389, 50.823194);
    ///
    /// let proj = FlatProjection::new(6., 51.);
    ///
    /// let flat_point = proj.project(lon, lat);
    /// ```
    pub fn project(&self, longitude: Decimal, latitude: Decimal) -> FlatPoint<Decimal> {
        let x = (longitude - self.lon) * self.kx;
        let y = (latitude - self.lat) * self.ky;

        FlatPoint { x, y }
    }

    /// Converts a [`FlatPoint`] back to a (lon, lat) tuple.
    ///
    /// [`FlatPoint`]: struct.FlatPoint.html
    ///
    /// ```
    /// # use flat_projection::FlatProjection;
    /// #
    /// let (lon, lat) = (6.186389, 50.823194);
    ///
    /// let proj = FlatProjection::new(6., 51.);
    ///
    /// let flat_point = proj.project(lon, lat);
    ///
    /// let result = proj.unproject(&flat_point);
    ///
    /// assert_eq!(result.0, lon);
    ///
    /// assert_eq!(result.1, lat);
    /// ```
    pub fn unproject(&self, p: &FlatPoint<Decimal>) -> (Decimal, Decimal) {
        (p.x / self.kx + self.lon, p.y / self.ky + self.lat)
    }
}

/// Representation of a geographical point on Earth as projected
/// by a [`FlatProjection`] instance.
///
/// [`FlatProjection`]: struct.FlatProjection.html
///
/// ```
/// # #[macro_use]
/// # extern crate assert_approx_eq;
/// #
/// # use flat_projection::FlatProjection;
/// #
/// # fn main() {
/// let (lon, lat) = (6.186389, 50.823194);
///
/// let proj = FlatProjection::new(6., 51.);
///
/// let flat_point = proj.project(lon, lat);
/// #
/// # assert_approx_eq!(flat_point.x, 13.0845f64, 0.001);
/// # assert_approx_eq!(flat_point.y, -19.6694f64, 0.001);
/// # }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct FlatPoint<Decimal> {
    /// X-axis component of the flat-surface point in kilometers
    pub x: Decimal,
    /// Y-axis component of the flat-surface point in kilometers
    pub y: Decimal,
}
impl From<FlatPoint<Decimal>> for Pos2D<Decimal> {
    fn from(flat_point: FlatPoint<Decimal>) -> Self {
        Pos2D::new(flat_point.x, flat_point.y)
    }
}
impl From<Pos2D<Decimal>> for FlatPoint<Decimal> {
    fn from(pos: Pos2D<Decimal>) -> Self {
        FlatPoint {
            x: pos.x(),
            y: pos.y(),
        }
    }
}
impl From<Pos3D<Decimal>> for FlatPoint<Decimal> {
    fn from(pos: Pos3D<Decimal>) -> Self {
        FlatPoint {
            x: pos.x(),
            y: pos.y(),
        }
    }
}
impl FlatPoint<Decimal> {
    /// Calculates the approximate distance in kilometers from
    /// this `FlatPoint` to another.
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate assert_approx_eq;
    /// # extern crate num_traits;
    /// # extern crate flat_projection;
    /// #
    /// # use num_traits::float::Float;
    /// # use flat_projection::FlatProjection;
    /// #
    /// # fn main() {
    /// let (lon1, lat1) = (6.186389, 50.823194);
    /// let (lon2, lat2) = (6.953333, 51.301389);
    ///
    /// let proj = FlatProjection::new(6.5, 51.05);
    ///
    /// let p1 = proj.project(lon1, lat1);
    /// let p2 = proj.project(lon2, lat2);
    ///
    /// let distance = p1.distance(&p2);
    /// // -> 75.648 km
    /// #
    /// # assert_approx_eq!(distance, 75.635_595, 0.02);
    /// # }
    /// ```
    pub fn distance(&self, other: &FlatPoint<Decimal>) -> Decimal {
        self.distance_squared(other).sqrt().unwrap()
    }

    /// Calculates the approximate squared distance from this `FlatPoint` to
    /// another.
    ///
    /// This method can be used for fast distance comparisons.
    pub fn distance_squared(&self, other: &FlatPoint<Decimal>) -> Decimal {
        let (dx, dy) = self.delta(other);
        distance_squared(dx, dy)
    }

    /// Calculates the approximate average bearing in degrees
    /// between -180 and 180 from this `FlatPoint` to another.
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate assert_approx_eq;
    /// # extern crate num_traits;
    /// # extern crate flat_projection;
    /// #
    /// # use num_traits::float::Float;
    /// # use flat_projection::FlatProjection;
    /// #
    /// # fn main() {
    /// let (lon1, lat1) = (6.186389, 50.823194);
    /// let (lon2, lat2) = (6.953333, 51.301389);
    ///
    /// let proj = FlatProjection::new(6.5, 51.05);
    ///
    /// let p1 = proj.project(lon1, lat1);
    /// let p2 = proj.project(lon2, lat2);
    ///
    /// let bearing = p1.bearing(&p2);
    /// // -> 45.3°
    /// #
    /// # assert_approx_eq!(bearing, 45.312, 0.001);
    /// # }
    /// ```
    pub fn bearing_unstable(&self, other: &FlatPoint<Decimal>) -> f64 {
        let (dx, dy) = self.delta(other);
        bearing_unstable(dx, dy)
    }

    /// Calculates the approximate [`distance`] and average [`bearing`]
    /// from this `FlatPoint` to another.
    ///
    /// [`distance`]: #method.distance
    /// [`bearing`]: #method.bearing
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate assert_approx_eq;
    /// # extern crate num_traits;
    /// # extern crate flat_projection;
    /// #
    /// # use num_traits::float::Float;
    /// # use flat_projection::FlatProjection;
    /// #
    /// # fn main() {
    /// let (lon1, lat1) = (6.186389, 50.823194);
    /// let (lon2, lat2) = (6.953333, 51.301389);
    ///
    /// let proj = FlatProjection::new(6.5, 51.05);
    ///
    /// let p1 = proj.project(lon1, lat1);
    /// let p2 = proj.project(lon2, lat2);
    ///
    /// let (distance, bearing) = p1.distance_bearing(&p2);
    /// // -> 75.648 km and 45.3°
    /// #
    /// # assert_approx_eq!(distance, 75.635_595, 0.02);
    /// # assert_approx_eq!(bearing, 45.312, 0.001);
    /// # }
    /// ```
    pub fn distance_bearing_unstable(&self, other: &FlatPoint<Decimal>) -> (Decimal, f64) {
        let (dx, dy) = self.delta(other);
        (
            distance_squared(dx, dy).sqrt().unwrap(),
            bearing_unstable(dx, dy),
        )
    }

    fn delta(&self, other: &FlatPoint<Decimal>) -> (Decimal, Decimal) {
        (self.x - other.x, self.y - other.y)
    }

    /// Returns a new `FlatPoint` given [`distance`] and [`bearing`] from this `FlatPoint`.
    ///
    /// [`distance`]: #method.distance (kilometers)
    /// [`bearing`]: #method.bearing (degrees)
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate assert_approx_eq;
    /// # extern crate num_traits;
    /// # extern crate flat_projection;
    /// #
    /// # use num_traits::float::Float;
    /// # use flat_projection::FlatProjection;
    /// #
    /// # fn main() {
    /// let (lon, lat) = (30.5, 50.5);
    ///
    /// let proj = FlatProjection::new(31., 50.);
    ///
    /// let p1 = proj.project(lon, lat);
    /// let (distance, bearing) = (1., 45.0);
    /// let p2 = p1.destination(distance, bearing);
    /// #
    /// # let res_distance = p1.distance(&p2);
    /// # let (dest_lon, dest_lat) = proj.unproject(&p2);
    /// #
    /// # assert_approx_eq!(dest_lon, 30.5098622, 0.00001);
    /// # assert_approx_eq!(dest_lat, 50.5063572, 0.00001);
    /// # }
    /// ```
    pub fn destination(&self, dist: Decimal, bearing: Decimal) -> FlatPoint<Decimal> {
        let a = bearing.to_radians();
        self.offset(a.sin() * dist, a.cos() * dist)
    }

    /// Returns a new `FlatPoint` given easting and northing offsets
    /// (in kilometers) from this `FlatPoint`.
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate assert_approx_eq;
    /// # extern crate num_traits;
    /// # extern crate flat_projection;
    /// #
    /// # use num_traits::float::Float;
    /// # use flat_projection::FlatProjection;
    /// #
    /// # fn main() {
    /// let (lon, lat) = (30.5, 50.5);
    ///
    /// let proj = FlatProjection::new(31., 50.);
    ///
    /// let p1 = proj.project(lon, lat);
    /// let p2 = p1.offset(10., 10.);
    /// #
    /// # let (dest_lon, dest_lat) = proj.unproject(&p2);
    /// # assert_approx_eq!(dest_lon, 30.6394736, 0.00001);
    /// # assert_approx_eq!(dest_lat, 50.5899044, 0.00001);
    /// # }
    /// ```
    pub fn offset(&self, dx: Decimal, dy: Decimal) -> FlatPoint<Decimal> {
        FlatPoint {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

fn distance_squared(dx: Decimal, dy: Decimal) -> Decimal {
    dx.powi(2) + dy.powi(2)
}

fn bearing_unstable(dx: Decimal, dy: Decimal) -> f64 {
    (-dx.to_f64().unwrap())
        .atan2(-dy.to_f64().unwrap())
        .to_degrees()
}

#[cfg(test)]
mod tests {
    use crate::proj::FlatProjection;
    use assert_approx_eq::assert_approx_eq;
    // use num_traits::Float;
    use rust_decimal::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn flatpoint_destination_ne() {
        let (lon, lat) = (dec!(30.5), dec!(50.5));
        let proj = FlatProjection::new(dec!(31), dec!(50));
        let p1 = proj.project(lon, lat);

        let (distance, bearing) = (dec!(1), dec!(45.0));
        let p2 = p1.destination(distance, bearing);
        let res_distance = p1.distance(&p2);
        let (dest_lon, dest_lat) = proj.unproject(&p2);
        assert_approx_eq!(dest_lon.to_f64().unwrap(), 30.5098622, 0.00001);
        assert_approx_eq!(dest_lat.to_f64().unwrap(), 50.5063572, 0.00001);
        assert_approx_eq!(
            distance.to_f64().unwrap(),
            res_distance.to_f64().unwrap(),
            0.00001
        );
    }

    #[test]
    fn flatpoint_destination_se() {
        let (lon, lat) = (dec!(30.5), dec!(50.5));
        let proj = FlatProjection::new(dec!(31), dec!(50));
        let p1 = proj.project(lon, lat);

        let (distance, bearing) = (dec!(1), dec!(135.0));

        let p2 = p1.destination(distance, bearing);
        let res_distance = p1.distance(&p2);
        let (dest_lon, dest_lat) = proj.unproject(&p2);
        assert_approx_eq!(dest_lon.to_f64().unwrap(), 30.5098622, 0.00001);
        assert_approx_eq!(dest_lat.to_f64().unwrap(), 50.4936427, 0.00001);
        assert_approx_eq!(
            distance.to_f64().unwrap(),
            res_distance.to_f64().unwrap(),
            0.00001
        );
    }

    #[test]
    fn flatpoint_destination_sw() {
        let (lon, lat) = (dec!(30.5), dec!(50.5));
        let proj = FlatProjection::new(dec!(31), dec!(50));
        let p1 = proj.project(lon, lat);

        let (distance, bearing) = (dec!(1), dec!(225.0));
        let p2 = p1.destination(distance, bearing);
        let res_distance = p1.distance(&p2);
        let (dest_lon, dest_lat) = proj.unproject(&p2);
        assert_approx_eq!(dest_lon.to_f64().unwrap(), 30.4901377, 0.00001);
        assert_approx_eq!(dest_lat.to_f64().unwrap(), 50.4936427, 0.00001);
        assert_approx_eq!(
            distance.to_f64().unwrap(),
            res_distance.to_f64().unwrap(),
            0.00001
        );
    }

    #[test]
    fn flatpoint_destination_nw() {
        let (lon, lat) = (dec!(30.5), dec!(50.5));
        let proj = FlatProjection::new(dec!(31), dec!(50));
        let p1 = proj.project(lon, lat);

        let (distance, bearing) = (dec!(1), dec!(315.0));
        let p2 = p1.destination(distance, bearing);
        let res_distance = p1.distance(&p2);
        let (dest_lon, dest_lat) = proj.unproject(&p2);
        assert_approx_eq!(dest_lon.to_f64().unwrap(), 30.4901377, 0.00001);
        assert_approx_eq!(dest_lat.to_f64().unwrap(), 50.5063572, 0.00001);
        assert_approx_eq!(
            distance.to_f64().unwrap(),
            res_distance.to_f64().unwrap(),
            0.00001
        );
    }
    #[test]
    fn it_works() {
        let aachen = (dec!(6.186389), dec!(50.823194));
        let meiersberg = (dec!(6.953333), dec!(51.301389));

        let average_longitude = (aachen.0 + meiersberg.0) / dec!(2);
        let average_latitude = (aachen.1 + meiersberg.1) / dec!(2);

        let proj = FlatProjection::new(average_longitude, average_latitude);

        let flat_aachen = proj.project(aachen.0, aachen.1);
        let flat_meiersberg = proj.project(meiersberg.0, meiersberg.1);

        let distance = flat_aachen.distance(&flat_meiersberg);
        let bearing = flat_aachen.bearing_unstable(&flat_meiersberg);

        const VINCENTY_DISTANCE: f64 = 75.635_595;
        assert_approx_eq!(distance.to_f64().unwrap(), VINCENTY_DISTANCE, 0.003);

        const VINCENTY_INITIAL_BEARING: f64 = 45.005_741;
        const VINCENTY_FINAL_BEARING: f64 = 45.602_300;
        const VINCENTY_AVERAGE_BEARING: f64 =
            (VINCENTY_INITIAL_BEARING + VINCENTY_FINAL_BEARING) / 2.;
        assert_approx_eq!(bearing, VINCENTY_AVERAGE_BEARING, 0.003);
    }
}
