use jortestkit::openssl::generate_keys;

use crate::common::{
    data,
    paths::BLOCK0_BIN,
    startup::{
        db::DbBuilder,
        server::{dump_settings, ServerBootstrapper, ServerSettingsBuilder},
    },
};
use assert_fs::TempDir;
use vit_servicing_station_lib::server::settings::Tls;

#[test]
pub fn secure_rest() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (token, hash) = data::token();

    let db_path = DbBuilder::new()
        .with_token(hash)
        .with_proposals(data::proposals())
        .build(&temp_dir)?;

    let (prv_key_file, cert_file) = generate_keys(&temp_dir);

    let tls = Tls {
        cert_file: Some(cert_file.to_str().unwrap().to_string()),
        priv_key_file: Some(prv_key_file.to_str().unwrap().to_string()),
    };

    let mut settings_builder: ServerSettingsBuilder = Default::default();
    let settings = settings_builder
        .with_random_localhost_address()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .with_tls_config(tls)
        .build();

    let server = ServerBootstrapper::new()
        .with_settings_file(dump_settings(&temp_dir, &settings))
        .start()?;

    let secured_rest_client = server.secure_rest_client_with_token(&token, &cert_file);
    assert!(secured_rest_client.health().is_ok());
    Ok(())
}
