mod handlers;
mod routes;

pub use routes::filter;

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::migrations;
    use crate::server::async_watch;
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use warp::hyper::StatusCode;
    use warp::{Filter, Reply};

    async fn get_voting_power<F>(voting_key: String, filter: &F) -> u64
    where
        F: Filter + 'static,
        F::Extract: Reply + Send,
    {
        let result = warp::test::request()
            .path(format!("/snapshot/{}", voting_key).as_ref())
            .reply(filter)
            .await;

        assert_eq!(result.status(), StatusCode::OK);
        let result_voting_power: u64 =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        result_voting_power
    }

    // the following is similar to what rsync does, which I think it is what we actually want to
    // do
    #[tokio::test]
    async fn test_snapshot_reloads_on_rename() {
        let shared_context = new_in_memmory_db_test_shared_context();

        let pool = &shared_context.read().await.db_connection_pool;
        migrations::initialize_db_with_migration(&pool.get().unwrap());

        let tmp_dir = tempfile::tempdir().unwrap();

        let content = serde_json::to_string(&[("1", u64::MAX.to_string())]).unwrap();

        let file_path = tmp_dir.path().join("snapshot.txt");
        tokio::fs::write(&file_path, content).await.unwrap();

        let _guard = async_watch(file_path.clone(), shared_context.clone())
            .await
            .unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let filter = routes::filter(snapshot_root, shared_context.clone());

        assert_eq!(u64::MAX, get_voting_power("1".to_string(), &filter).await);

        let content = serde_json::to_string(&[("1", "2"), ("3", "3")]).unwrap();
        let mut tmp_file_path = file_path.clone();
        tmp_file_path.set_file_name("snapshot.json.tmp");

        tokio::fs::write(&tmp_file_path, content).await.unwrap();
        tokio::fs::rename(&tmp_file_path, file_path).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        assert_eq!(2u64, get_voting_power("1".to_string(), &filter).await);
        assert_eq!(3u64, get_voting_power("3".to_string(), &filter).await);
    }
}
