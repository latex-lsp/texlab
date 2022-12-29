use salsa::{DbWithJar, ParallelDatabase};
use threadpool::ThreadPool;

use crate::{Database, Db};

#[derive(Default)]
pub struct Engine {
    db: Database,
    pool: ThreadPool,
}

impl Engine {
    pub fn read(&self) -> &dyn Db {
        &self.db
    }

    pub fn write(&mut self) -> &mut dyn Db {
        self.pool.join();
        &mut self.db
    }

    pub fn fork<F: FnOnce(&dyn Db) + Send + 'static>(&self, action: F) {
        let snapshot = self.db.snapshot();
        self.pool.execute(move || {
            action(snapshot.as_jar_db());
        });
    }

    pub fn finish(self) {
        self.pool.join();
    }
}
