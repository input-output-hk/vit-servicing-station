#[cfg(test)]
mod test {
    use crate::db::migrations;
    use crate::server::async_watch;
    use crate::server::snapshot_watcher::VoterHIR;
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use jormungandr_lib::crypto::account::Identifier;
    use tracing::Level;
    use warp::hyper::StatusCode;
    use warp::{Filter, Reply};

    async fn get_voting_power<F>(
        tag: &str,
        voting_key: &str,
        filter: &F,
    ) -> Result<Vec<(u64, String)>, StatusCode>
    where
        F: Filter + 'static,
        F::Extract: Reply + Send,
    {
        let result = warp::test::request()
            .path(format!("/snapshot/{}/{}", tag, voting_key).as_ref())
            .reply(filter)
            .await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            return Err(status);
        }

        let result_voting_power: Vec<serde_json::Value> =
            serde_json::from_str(dbg!(&String::from_utf8(result.body().to_vec()).unwrap()))
                .unwrap();

        Ok(result_voting_power
            .into_iter()
            .map(|v| {
                (
                    v["voting_power"].as_u64().unwrap(),
                    v["voting_group"].as_str().unwrap().to_string(),
                )
            })
            .collect::<Vec<_>>())
    }

    // the following is similar to what rsync does, which I think it is what we actually want to
    // do
    #[tokio::test]
    async fn test_snapshot_reloads_on_rename() {
        const DAILY: &str = "daily";
        const GROUP: &str = "group";

        let _e = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(tracing_subscriber::fmt::TestWriter::new())
            .try_init();

        let keys = [
            "0000000000000000000000000000000000000000000000000000000000000000",
            "1111111111111111111111111111111111111111111111111111111111111111",
        ];

        let shared_context = new_in_memmory_db_test_shared_context();

        let pool = &shared_context.read().await.db_connection_pool;
        migrations::initialize_db_with_migration(&pool.get().unwrap());

        let tmp_dir = tempfile::tempdir().unwrap();

        let content = serde_json::to_string(&[VoterHIR {
            voting_key: Identifier::from_hex(keys[0]).unwrap(),
            voting_group: GROUP.to_string(),
            voting_power: u64::MAX.into(),
        }])
        .unwrap();

        let file_path = tmp_dir.path().join(format!("{}-snapshot.json", DAILY));
        tokio::fs::write(&file_path, content).await.unwrap();

        let (shared_context, update_handler) = snapshot_service::new_context().unwrap();
        let _guard = async_watch(tmp_dir.path().to_path_buf(), update_handler)
            .await
            .unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let filter = snapshot_service::filter(snapshot_root, shared_context.clone());

        assert_eq!(
            get_voting_power("daily", keys[0], &filter).await.unwrap(),
            vec![(u64::MAX, GROUP.to_string())]
        );

        let content = serde_json::to_string(&[
            VoterHIR {
                voting_key: Identifier::from_hex(keys[0]).unwrap(),
                voting_group: "group".to_string(),
                voting_power: 2.into(),
            },
            VoterHIR {
                voting_key: Identifier::from_hex(keys[1]).unwrap(),
                voting_group: "group".to_string(),
                voting_power: 3.into(),
            },
        ])
        .unwrap();

        let mut tmp_file_path = file_path.clone();
        tmp_file_path.set_file_name("daily-snapshot.json.tmp");

        tokio::fs::write(&tmp_file_path, content).await.unwrap();
        tokio::fs::rename(&tmp_file_path, file_path).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        assert_eq!(
            get_voting_power(DAILY, keys[0], &filter).await.unwrap(),
            vec![(2, GROUP.to_string())]
        );
        assert_eq!(
            get_voting_power(DAILY, keys[1], &filter).await.unwrap(),
            vec![(3, GROUP.to_string())]
        );
    }

    #[tokio::test]
    async fn test_snapshot_get_tags() {
        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        let _e = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(tracing_subscriber::fmt::TestWriter::new())
            .try_init();

        let keys = [
            "0000000000000000000000000000000000000000000000000000000000000000",
            "1111111111111111111111111111111111111111111111111111111111111111",
        ];

        let shared_context = new_in_memmory_db_test_shared_context();

        let pool = &shared_context.read().await.db_connection_pool;
        migrations::initialize_db_with_migration(&pool.get().unwrap());

        let tmp_dir = tempfile::tempdir().unwrap();

        let content_a = serde_json::to_string(&[
            VoterHIR {
                voting_key: Identifier::from_hex(keys[0]).unwrap(),
                voting_group: GROUP1.to_string(),
                voting_power: 1.into(),
            },
            VoterHIR {
                voting_key: Identifier::from_hex(keys[0]).unwrap(),
                voting_group: GROUP2.to_string(),
                voting_power: 2.into(),
            },
        ])
        .unwrap();

        let content_b = serde_json::to_string(&[VoterHIR {
            voting_key: Identifier::from_hex(keys[0]).unwrap(),
            voting_group: GROUP1.to_string(),
            voting_power: 2.into(),
        }])
        .unwrap();

        let file_path_a = tmp_dir.path().join(format!("tag_a-snapshot.json"));
        let file_path_b = tmp_dir.path().join(format!("tag_b-snapshot.json"));

        tokio::fs::write(&file_path_a, content_a).await.unwrap();
        tokio::fs::write(&file_path_b, content_b).await.unwrap();

        let (shared_context, update_handler) = snapshot_service::new_context().unwrap();
        let _guard = async_watch(tmp_dir.path().to_path_buf(), update_handler)
            .await
            .unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let filter = snapshot_service::filter(snapshot_root, shared_context.clone());

        assert_eq!(
            get_voting_power("tag_a", keys[0], &filter).await.unwrap(),
            vec![(1u64, GROUP1.to_string()), (2u64, GROUP2.to_string())]
        );

        assert_eq!(
            get_voting_power("tag_b", keys[0], &filter).await.unwrap(),
            vec![(2u64, GROUP1.to_string())]
        );

        assert!(get_voting_power("tag_c", keys[0], &filter).await.is_err());

        let result = warp::test::request()
            .path(format!("/snapshot").as_ref())
            .reply(&filter)
            .await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            todo!();
        }

        let mut tags: Vec<String> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        tags.sort_unstable();

        assert_eq!(tags, vec!["tag_a", "tag_b"]);
    }
}
