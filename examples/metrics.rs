#[cfg(feature = "metrics-exporter")]
fn main() -> Result<(), Box<Error>> {
    use metrics::{histogram, increment};
    use rustmann::RiemannObserverBuilder;

    let builder = RiemannObserverBuilder::new().host("localhost");

    Ok(())
}

#[cfg(not(feature = "metrics-exporter"))]
fn main() -> Result<(), Box<Error>> {
    println!("Please enable feature metrics-exporter to use this example.");
    Ok(())
}
