use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;

    // Create a namespace using the basic helper (available when vector feature is disabled)
    #[cfg(not(feature = "vector"))]
    liath.create_namespace_basic("docs")?;

    // If vector feature is enabled, create with explicit parameters
    #[cfg(feature = "vector")]
    {
        liath.create_namespace("docs", 128, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;
    }

    liath.put("docs", b"hello", b"world")?;
    let value = liath.get("docs", b"hello")?;
    println!("value = {:?}", value.map(|v| String::from_utf8_lossy(&v).into_owned()));
    Ok(())
}
