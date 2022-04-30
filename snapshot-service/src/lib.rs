mod handlers;
mod routes;

use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
pub use routes::filter;
use sled::Transactional;
use std::cmp::max;
use voting_hir::VoterHIR;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DbError(#[from] sled::Error),

    #[error(transparent)]
    DbTxError(#[from] sled::transaction::TransactionError),

    #[error("internal error")]
    InternalError,
}

pub type Tag = String;
type Group = String;

#[derive(Clone)]
pub struct SharedContext {
    db: sled::Db,
}

impl SharedContext {
    #[tracing::instrument(skip(self))]
    pub fn get_voting_power(
        &self,
        tag: &str,
        id: &Identifier,
    ) -> Result<Option<Vec<(Group, Value)>>, Error> {
        let entries = self.db.open_tree("entries")?;
        let tags = self.db.open_tree("tags")?;

        let tag = if let Some(tag) = tags.get(tag)? {
            tag
        } else {
            return Ok(None);
        };

        let key = {
            let mut key = [0u8; 32 + 4];

            let (tag_part, id_part) = key.split_at_mut(4);
            tag_part.copy_from_slice(&*tag);

            id_part.copy_from_slice(id.as_ref().as_ref());

            key
        };

        let key_len = key.len();

        let mut result: Vec<(Group, Value)> = vec![];

        for entries in entries.range(key.clone()..) {
            let (k, v) = entries?;

            if k[0..key_len] > key[..] {
                break;
            }

            let group =
                String::from_utf8(k[key_len..].to_vec()).map_err(|_| Error::InternalError)?;

            let voting_power = Value::from(u64::from_be_bytes(v.as_ref().try_into().unwrap()));

            result.push((group, voting_power));
        }

        Ok(Some(result))
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>, Error> {
        let tags = self.db.open_tree("tags")?;

        let mut result = vec![];
        for entries in tags.iter() {
            let (tag, _) = entries?;
            result.push(String::from_utf8(tag.to_vec()).map_err(|_| Error::InternalError)?);
        }

        Ok(result)
    }
}

// do NOT implement/derive Clone for this. The implementation of update relies on &mut self and the
// split in a reader type and a writer type is to enforce a single writer.
pub struct UpdateHandle {
    _db: sled::Db,
    tags: sled::Tree,
    entries: sled::Tree,
    next_tag_id: u32,
}

impl UpdateHandle {
    pub fn new(db: sled::Db) -> Result<Self, Error> {
        let tags = db.open_tree("tags")?;
        let entries = db.open_tree("entries")?;

        let mut next_tag_id = None;

        for kv in tags.iter() {
            let (_, v) = kv?;

            let id = u32::from_be_bytes(v.as_ref().try_into().unwrap());

            next_tag_id = next_tag_id.map(|c| max(c, id)).or_else(|| Some(id));
        }

        let next_tag_id = next_tag_id.map(|id| id + 1).unwrap_or(0);

        Ok(UpdateHandle {
            _db: db,
            tags,
            entries,
            next_tag_id,
        })
    }

    #[tracing::instrument(skip(self, snapshot))]
    pub async fn update(
        &mut self,
        tag: &str,
        snapshot: impl IntoIterator<Item = VoterHIR>,
    ) -> Result<(), Error> {
        let mut batch = sled::Batch::default();

        let tag_id = match self.tags.get(tag.clone())? {
            Some(tag_id_raw) => {
                let tag_id = u32::from_be_bytes(tag_id_raw.as_ref().try_into().unwrap());

                for entry in self
                    .entries
                    .range(tag_id_raw.as_ref()..&(tag_id + 1).to_be_bytes())
                {
                    let (k, _) = entry?;

                    batch.remove(k);
                }

                Some(tag_id_raw)
            }
            None => None,
        };

        for entry in snapshot.into_iter() {
            let VoterHIR {
                voting_key,
                voting_group,
                voting_power,
            } = entry;

            let mut key = Vec::with_capacity(4 + 32 + voting_group.as_bytes().len());

            match tag_id {
                None => key.extend(&self.next_tag_id.to_be_bytes()),
                Some(ref tag_id_raw) => key.extend(&**tag_id_raw),
            }

            key.extend(voting_key.as_ref().as_ref());
            key.extend(voting_group.as_bytes());

            batch.insert(key, &u64::from(voting_power).to_be_bytes());
        }

        {
            let tag = tag.to_string();
            let tag_id = tag_id.clone();
            let tags = self.tags.clone();
            let entries = self.entries.clone();
            let next_tag_id = self.next_tag_id.to_be_bytes();

            tokio::task::spawn_blocking(move || {
                (&tags, &entries).transaction(move |(tags, entries)| {
                    if let None = tag_id {
                        tags.insert(tag.as_bytes(), &next_tag_id)?;
                    }

                    entries.apply_batch(&batch)?;
                    Ok(())
                })?;

                Ok(())
            })
            .await
            .unwrap()
            .map_err(Error::DbTxError)?;
        }

        if let None = tag_id {
            self.next_tag_id += 1;
        }

        Ok(())
    }
}

pub fn new_context() -> Result<(SharedContext, UpdateHandle), Error> {
    let db = sled::Config::new().temporary(true).open()?;

    Ok((SharedContext { db: db.clone() }, UpdateHandle::new(db)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn test_snapshot() {
        let (rx, mut tx) = new_context().unwrap();

        let keys = [
            Identifier::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            Identifier::from_hex(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
        ];

        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        let key_0_values = [
            (GROUP1.to_string(), Value::from(1)),
            (GROUP2.to_string(), Value::from(2)),
        ];

        let content_a = std::iter::repeat(keys[0].clone())
            .take(key_0_values.len())
            .zip(key_0_values.iter().cloned())
            .map(|(voting_key, (voting_group, voting_power))| VoterHIR {
                voting_key,
                voting_group,
                voting_power,
            })
            .collect::<Vec<_>>();

        tx.update(TAG1, content_a.clone()).await.unwrap();

        let key_1_values = [(GROUP1.to_string(), Value::from(3))];

        let content_b = std::iter::repeat(keys[1].clone())
            .take(key_1_values.len())
            .zip(key_1_values.iter().cloned())
            .map(|(voting_key, (voting_group, voting_power))| VoterHIR {
                voting_key,
                voting_group,
                voting_power,
            })
            .collect::<Vec<_>>();

        tx.update(TAG2, [content_a, content_b].concat())
            .await
            .unwrap();

        assert_eq!(
            &key_0_values[..],
            &rx.get_voting_power(TAG1, &keys[0]).unwrap().unwrap()[..],
        );

        assert!(&rx
            .get_voting_power(TAG1, &keys[1])
            .unwrap()
            .unwrap()
            .is_empty(),);

        assert_eq!(
            &key_1_values[..],
            &rx.get_voting_power(TAG2, &keys[1]).unwrap().unwrap()[..],
        );
    }
}
