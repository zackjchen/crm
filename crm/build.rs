use std::fs;

fn main() -> anyhow::Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();

    builder
        .out_dir("src/pb")
        .compile_protos(&["../protos/crm/crm.proto"], &["../protos"])?;

    Ok(())
}
