fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/zkrpc.proto"], &["proto"])?;
    tonic_build::configure().compile(&["proto/mesh.proto"], &["proto"])?;
    Ok(())
}
