use crate::common::startup::quick_start;
use askama::Template;
use assert_fs::TempDir;
use pretty_assertions::assert_eq;

#[test]
pub fn get_challenge_by_id_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let challenge_id: i32 = snapshot.challenges().first().unwrap().id;

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());

    let challenge = graphql_client.challenge_by_id(challenge_id).unwrap();
    assert_eq!(
        challenge,
        snapshot.challenge_by_id(challenge_id).unwrap().clone()
    );
    Ok(())
}

#[test]
pub fn challenges_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();
    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let mut challenges = snapshot.challenges();
    challenges.sort_by_key(|k| k.id);
    assert_eq!(graphql_client.challenges().unwrap(), challenges);
    Ok(())
}
