fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile_protos(&["proto/oceanus/v1/study_search.proto"], &["proto"])?;
    Ok(())
}
