use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;
fn main() -> anyhow::Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();

    builder
        .out_dir("src/pb")
        .with_type_attributes(&["MaterializeRequest"], &[r#"#[derive(Eq, Hash)]"#])
        // .with_serde(&["Content", "ContentType", "Publisher"], true, true, Some(&[r#"#[serde(rename_all = "camelCase")]"#]))
        .compile_protos(
            &[
                "../protos/metadata/messages.proto",
                "../protos/metadata/rpc.proto",
            ],
            &["../protos"],
        )
        .unwrap();

    Ok(())
}
