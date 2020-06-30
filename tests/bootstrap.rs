use assert_fs::TempDir;
use vit_servicing_station::testing::{
    get_testing_token,
    startup::{db::DbBuilder, server::Starter},
};

#[test]
pub fn bootstrap() {
    let mut temp_dir = TempDir::new().unwrap();
    temp_dir = temp_dir.into_persistent();

    let (token, hash) = get_testing_token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_migrations_from("migrations")
        .build(&temp_dir)
        .unwrap();

    let server = Starter::new()
        .with_localhost_address(3030)
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path("resources\\tests\\block0.bin")
        .start()
        .unwrap();

    let mut client = server.rest_client();

    client.set_api_token(hash);
    println!("{:?}", client.funds());
}
