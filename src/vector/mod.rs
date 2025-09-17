#[cfg(feature = "vector")]
mod usearch_wrapper;

#[cfg(feature = "vector")]
pub use usearch_wrapper::UsearchWrapper;

#[cfg(not(feature = "vector"))]
pub struct UsearchWrapper;

#[cfg(not(feature = "vector"))]
impl UsearchWrapper {
    pub fn new(_dimensions: usize, _metric: (), _scalar: ()) -> anyhow::Result<Self> { Ok(Self) }
    pub fn reserve(&self, _capacity: usize) -> anyhow::Result<()> { Ok(()) }
    pub fn add(&self, _id: u64, _vector: &[f32]) -> anyhow::Result<()> { Ok(()) }
    pub fn search(&self, _vector: &[f32], _k: usize) -> anyhow::Result<Vec<(u64, f32)>> {
        anyhow::bail!("vector feature is disabled")
    }
}
