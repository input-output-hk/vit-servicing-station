mod handlers;
mod routes;

use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
pub use routes::filter;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::sync::RwLock;
use voting_hir::VoterHIR;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

pub type Tag = String;
type Group = String;

#[derive(Default)]
pub struct Db {
    tags: BTreeMap<Tag, HashMap<Identifier, BTreeMap<Group, Value>>>,
}

#[derive(Clone)]
pub struct SharedContext {
    db: Arc<RwLock<Db>>,
}

impl SharedContext {
    pub async fn get_voting_power(&self, tag: Tag, id: Identifier) -> Option<Vec<(Group, Value)>> {
        self.db
            .read()
            .await
            .tags
            .get(&tag)
            .and_then(|m| m.get(&id))
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    pub async fn get_tags(&self) -> Vec<Tag> {
        self.db.read().await.tags.keys().cloned().collect()
    }
}

pub struct UpdateHandler {
    db: Arc<RwLock<Db>>,
}

impl UpdateHandler {
    pub async fn update(&self, tag: String, snapshot: Vec<VoterHIR>) {
        let mut db = self.db.write().await;

        let updated = snapshot.into_iter().fold(
            HashMap::<Identifier, BTreeMap<Group, Value>>::new(),
            |mut map,
             VoterHIR {
                 voting_key,
                 voting_group,
                 voting_power,
             }| {
                map.entry(voting_key)
                    .or_insert(Default::default())
                    .insert(voting_group, voting_power);

                map
            },
        );

        if updated.len() > 0 {
            db.tags.insert(tag, updated);
        } else {
            db.tags.remove(&tag);
        }
    }
}

pub fn new_context() -> (SharedContext, UpdateHandler) {
    let db = Db::default();
    let arc = Arc::new(RwLock::new(db));

    (
        SharedContext {
            db: Arc::clone(&arc),
        },
        UpdateHandler { db: arc },
    )
}
