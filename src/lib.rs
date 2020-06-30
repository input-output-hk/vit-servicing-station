#[macro_use]
extern crate diesel;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(test)] {
        #[macro_use]
        extern crate diesel_migrations;
        pub mod testing;
    } else if #[cfg(feature = "test-api")] {
        extern crate diesel_migrations;
        pub mod testing;
    }
}

pub mod db;
pub mod server;
pub mod server_settings;
pub mod utils;
pub mod v0;
