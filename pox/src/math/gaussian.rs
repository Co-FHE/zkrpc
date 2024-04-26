use types::FixedPoint;

trait Kernel {}

enum Gaussian {
    Vanilla,
    Taylor,
}
struct GaussianParams<T: FixedPoint> {
    mean: T,
    variance: T,
}
impl Kernel for Gaussian {}
impl Gaussian {
    fn new() -> Self {
        Gaussian::Vanilla
    }
}
