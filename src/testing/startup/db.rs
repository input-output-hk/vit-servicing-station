use crate::db::{models::api_tokens::APITokenData, schema::api_tokens};
use assert_fs::{fixture::PathChild, TempDir};
use diesel::{connection::Connection, prelude::*};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

pub struct DbBuilder {
    migrations_folder: Option<PathBuf>,
    token: Option<APITokenData>,
}

impl DbBuilder {
    pub fn new() -> Self {
        Self {
            migrations_folder: None,
            token: None,
        }
    }

    pub fn with_token(&mut self, token: APITokenData) -> &mut Self {
        self.token = Some(token);
        self
    }

    pub fn with_migrations_from<P: AsRef<Path>>(&mut self, migrations_folder: P) -> &mut Self {
        self.migrations_folder = Some(migrations_folder.as_ref().into());
        self
    }

    fn create_db_if_not_exists(&self, db_path: &str) -> Result<(), DbBuilderError> {
        rusqlite::Connection::open(db_path).map_err(DbBuilderError::CannotCreateDatabase)?;
        Ok(())
    }

    fn do_migration(
        &self,
        connection: &SqliteConnection,
        migration_folder: &PathBuf,
    ) -> Result<(), DbBuilderError> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        diesel_migrations::run_pending_migrations_in_directory(
            connection,
            migration_folder,
            &mut handle,
        )
        .map_err(DbBuilderError::MigrationsError)
    }

    fn try_do_migration(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(migrations_folder) = &self.migrations_folder {
            self.do_migration(&connection, migrations_folder)?;
        }
        Ok(())
    }

    fn try_insert_token(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(token) = &self.token {
            let values = (
                api_tokens::dsl::token.eq(&(*token.token.as_ref())),
                api_tokens::dsl::creation_time.eq(token.creation_time),
                api_tokens::dsl::expire_time.eq(token.expire_time),
            );

            diesel::insert_into(api_tokens::table)
                .values(values)
                .execute(connection)
                .map_err(DbBuilderError::DieselError)?;
        }
        Ok(())
    }

    pub fn build(&self, temp_dir: &TempDir) -> Result<PathBuf, DbBuilderError> {
        let db = temp_dir.child("vit_station.db");
        let db_path = db
            .path()
            .to_str()
            .ok_or_else(|| DbBuilderError::CannotExtractTempPath)?;
        println!("Building db in {:?}...", db_path);

        self.create_db_if_not_exists(db_path)?;

        let connection = SqliteConnection::establish(db_path).unwrap();
        self.try_do_migration(&connection)?;
        self.try_insert_token(&connection)?;
        Ok(PathBuf::from(db.path()))
    }
}

impl Default for DbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug)]
pub enum DbBuilderError {
    #[error("internal diesel error")]
    DieselError(#[from] diesel::result::Error),
    #[error("Cannot open or create database")]
    CannotCreateDatabase(#[from] rusqlite::Error),
    #[error("Cannot initialize on temp directory")]
    CannotExtractTempPath,
    #[error("migration errors")]
    MigrationsError(#[from] diesel::migration::RunMigrationsError),
}
