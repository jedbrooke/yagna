#![allow(dead_code)] // Crate under development
#![allow(unused_variables)] // Crate under development
use crate::processor::PaymentProcessor;
use ya_payment_driver::{DummyDriver, PaymentDriver};
use ya_persistence::executor::DbExecutor;
use ya_service_api_interfaces::*;

#[macro_use]
extern crate diesel;

pub mod api;
pub mod dao;
pub mod error;
pub mod models;
pub mod processor;
pub mod schema;
pub mod service;
pub mod utils;

pub mod migrations {
    #[derive(diesel_migrations::EmbedMigrations)]
    struct _Dummy;
}

fn payment_driver_factory(_db: &DbExecutor) -> impl PaymentDriver {
    DummyDriver::new()
}

pub struct PaymentService;

impl Service for PaymentService {
    type Cli = ();
}

impl PaymentService {
    pub async fn gsb<Context: Provider<Self, DbExecutor>>(context: &Context) -> anyhow::Result<()> {
        let db: DbExecutor = context.component();
        db.apply_migration(migrations::run_with_output)?;
        let driver = payment_driver_factory(&db);
        let processor = PaymentProcessor::new(driver, db.clone());
        self::service::bind_service(&db, processor);
        Ok(())
    }

    pub fn rest(db: &DbExecutor) -> actix_web::Scope {
        api::web_scope(db)
    }
}
