use std::fs;

use proto_builder_trait::tonic::BuilderAttributes;

fn main() -> anyhow::Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();

    builder
        .out_dir("src/pb")
        .with_derive_builder(&["WelcomeRequest", "RecallRequest", "RemindRequest"], None)
        .compile_protos(
            &["../protos/crm/messages.proto", "../protos/crm/rpc.proto"],
            &["../protos"],
        )?;

    Ok(())
}
