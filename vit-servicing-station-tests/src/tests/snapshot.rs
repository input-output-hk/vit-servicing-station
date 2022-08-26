use crate::common::{
    clients::RawRestClient,
    snapshot::{Snapshot, SnapshotBuilder, SnapshotInfoUpdate, SnapshotUpdater, VoterInfo},
    startup::quick_start,
};
use assert_fs::TempDir;

#[test]
pub fn import_new_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = Snapshot::default();

    rest_client.put_snapshot(&snapshot).unwrap();

    assert_eq!(
        vec![snapshot.tag.to_string()],
        rest_client.snapshot_tags().unwrap(),
        "expected tags vs tags taken from REST API"
    );

    for (idx, entry) in snapshot.content.snapshot.iter().enumerate() {
        let voter_info = VoterInfo::from(entry.clone());
        let voter_info_update = rest_client
            .voter_info(&snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voter_info],
            voter_info_update.voter_info,
            "wrong voting info for entry idx: {}",
            idx
        );
        assert_eq!(
            snapshot.content.update_timestamp, voter_info_update.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn reimport_with_empty_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = Snapshot::default();

    rest_client.put_snapshot(&snapshot).unwrap();

    let empty_snapshot = Snapshot {
        tag: snapshot.tag.clone(),
        content: SnapshotInfoUpdate {
            snapshot: Vec::new(),
            update_timestamp: 0,
        },
    };

    rest_client.put_snapshot(&empty_snapshot).unwrap();
    for (idx, entry) in snapshot.content.snapshot.iter().enumerate() {
        assert!(
            rest_client
                .voter_info(&snapshot.tag, &entry.hir.voting_key.to_hex())
                .unwrap()
                .voter_info
                .is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
    }
}
#[test]
pub fn replace_snapshot_with_tag() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_snapshot = Snapshot::default();

    rest_client.put_snapshot(&first_snapshot).unwrap();

    let second_snapshot = Snapshot::default();

    rest_client.put_snapshot(&second_snapshot).unwrap();
    for (idx, entry) in first_snapshot.content.snapshot.iter().enumerate() {
        let outdated_voter_info = rest_client
            .voter_info(&first_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert!(
            outdated_voter_info.voter_info.is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            first_snapshot.content.update_timestamp, outdated_voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
    for (idx, entry) in second_snapshot.content.snapshot.iter().enumerate() {
        let voter_info = VoterInfo::from(entry.clone());
        let voter_info_update = rest_client
            .voter_info(&second_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voter_info],
            voter_info_update.voter_info,
            "wrong voting info for entry idx: {}",
            idx
        );
        assert_eq!(
            second_snapshot.content.update_timestamp, voter_info_update.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn import_snapshots_with_different_tags() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_snapshot = Snapshot::default();

    rest_client.put_snapshot(&first_snapshot).unwrap();

    let second_snapshot = SnapshotUpdater::from(first_snapshot.clone())
        .with_tag("fund9")
        .build();

    rest_client.put_snapshot(&second_snapshot).unwrap();

    for (idx, entry) in first_snapshot.content.snapshot.iter().enumerate() {
        let voter_info = VoterInfo::from(entry.clone());

        let first_voter_info_update = rest_client
            .voter_info(&first_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voter_info.clone()],
            first_voter_info_update.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            first_snapshot.content.update_timestamp, first_voter_info_update.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );

        let second_voter_info_update = rest_client
            .voter_info(&second_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voter_info],
            second_voter_info_update.voter_info,
            "wrong data for entry idx: {}",
            idx
        );

        assert_eq!(
            second_snapshot.content.update_timestamp, second_voter_info_update.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn import_malformed_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client: RawRestClient = server.rest_client_with_token(&data.token_hash()).into();

    let snapshot = Snapshot::default();
    let mut content = serde_json::to_string(&snapshot.content).unwrap();
    content.pop();
    assert!(rest_client
        .put_snapshot(&snapshot.tag, content)
        .unwrap()
        .status()
        .is_client_error());
}
#[test]
pub fn import_big_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = SnapshotBuilder::default()
        .with_tag("big")
        .with_entries_count(100_000)
        .with_groups(vec![
            "drep".to_string(),
            "direct".to_string(),
            "drep2".to_string(),
            "drep3".to_string(),
        ])
        .build();

    rest_client.put_snapshot(&snapshot).unwrap();
    let entry = snapshot.content.snapshot[0].clone();
    let voter_info = VoterInfo::from(entry.clone());
    let voter_info_update = rest_client
        .voter_info(&snapshot.tag, &entry.hir.voting_key.to_hex())
        .unwrap();

    assert_eq!(
        vec![voter_info],
        voter_info_update.voter_info,
        "wrong data for entry 0"
    );
    assert_eq!(
        snapshot.content.update_timestamp, voter_info_update.last_updated,
        "wrong timestamp for entry 0"
    );
}
