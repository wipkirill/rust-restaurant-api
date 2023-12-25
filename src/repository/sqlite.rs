use std::sync::{Mutex, MutexGuard};
use crate::domain::types::{
    IdType, Item, ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, QuantityType, TableId,
    VersionType,
};
use crate::repository::*;
use rusqlite::{params, params_from_iter, Connection, Error::SqliteFailure};

// An Sqlite repository implementation

pub struct SqliteRepository {
    pub connection: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn try_new(path: &str) -> Result<Self, ()> {
        let connection = match Connection::open(path) {
            Ok(connection) => connection,
            _ => return Err(()),
        };


        match connection.execute(
            "CREATE TABLE IF NOT EXISTS item (
                item_id      INTEGER NOT NULL,
                table_id     INTEGER NOT NULL,
                name         TEXT NOT NULL,
                notes        TEXT,
                quantity     INTEGER,
                version      INTEGER,
                deleted      INTEGER, 
                time_to_prepare TEXT
            )",
            [],
        ) {
            Ok(_) => {},
            Err(_) => return Err(()),
        }

        match connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx1 ON item(item_id, table_id) WHERE deleted = 0",
            [],
        ) {
            Ok(_) => Ok(Self {
                connection: Mutex::new(connection),
            }),
            _ => Err(()),
        }
    }

    fn fetch_item_rows(
        lock: &MutexGuard<'_, Connection>,
        table_id: IdType,
        item_id: Option<IdType>,
        include_deleted: bool
    ) -> Result<
        Vec<(
            IdType,
            IdType,
            String,
            String,
            QuantityType,
            bool,
            VersionType,
            String,
        )>,
        (),
    > {
        let (query, params) = match item_id {
            Some(item_id) => {
                match include_deleted {
                    true => ("select item_id, table_id, name, notes, quantity, deleted, version, time_to_prepare from item where item_id = ? and table_id= ?",
                        vec![item_id, table_id]),
                    false=>("select item_id, table_id, name, notes, quantity, deleted, version, time_to_prepare from item where item_id = ? and table_id= ? and deleted=0",
                        vec![item_id, table_id]),
            }
        },
            _ => {
                match include_deleted {
                    true => ("select item_id, table_id, name, notes, quantity, deleted, version, time_to_prepare from item where table_id = ?", vec![table_id]),
                    false => ("select item_id, table_id, name, notes, quantity, deleted, version, time_to_prepare from item where table_id = ? and deleted=0", vec![table_id])
                }
            },
        };

        let mut stmt = match lock.prepare(query) {
            Ok(stmt) => stmt,
            _ => return Err(()),
        };

        let mut rows = match stmt.query(params_from_iter(params)) {
            Ok(rows) => rows,
            _ => return Err(()),
        };

        let mut item_rows = vec![];

        while let Ok(Some(row)) = rows.next() {
            match (
                row.get::<usize, IdType>(0),
                row.get::<usize, IdType>(1),
                row.get::<usize, String>(2),
                row.get::<usize, String>(3),
                row.get::<usize, QuantityType>(4),
                row.get::<usize, bool>(5),
                row.get::<usize, VersionType>(6),
                row.get::<usize, String>(7),
            ) {
                (
                    Ok(item_id),
                    Ok(table_id),
                    Ok(name),
                    Ok(notes),
                    Ok(quantity),
                    Ok(deleted),
                    Ok(version),
                    Ok(time_to_prepare),
                ) => item_rows.push((
                    item_id,
                    table_id,
                    name,
                    notes,
                    quantity,
                    deleted,
                    version,
                    time_to_prepare,
                )),
                _ => return Err(()),
            };
        }

        Ok(item_rows)
    }
}

impl Repository for SqliteRepository {
    fn insert(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
        item_name: ItemName,
        item_notes: ItemNotes,
        item_quantity: ItemQuantity<QuantityType>,
        item_deleted: bool,
        item_version: ItemVersion<VersionType>,
        item_time_to_prepare: String,
    ) -> Result<Item, InsertError> {
        let mut lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(InsertError::Unknown),
        };

        let transaction = match lock.transaction() {
            Ok(transaction) => transaction,
            _ => return Err(InsertError::Unknown),
        };
        let deleted_int = match item_deleted {
            true => 1,
            false => 0
        };

        match transaction.execute(
            "insert into item (item_id, table_id, name, notes, quantity, deleted, version, time_to_prepare) values (?,?,?,?,?,?,?,?)",
            params![IdType::from(item_id), IdType::from(table_id), String::from(item_name.clone()),
            String::from(item_notes.clone()), QuantityType::from(item_quantity.clone()), deleted_int, VersionType::from(item_version.clone()),
            item_time_to_prepare],
        ) {
            Ok(_) => {}
            Err(SqliteFailure(_, Some(message))) => {
                if message.contains("UNIQUE constraint failed") {
                    return Err(InsertError::Conflict);
                } else {
                    println!("Message insert {}", message);
                    return Err(InsertError::Unknown);
                }
            }
            _ => return Err(InsertError::Unknown),
        };

        match transaction.commit() {
            Ok(_) => Ok(Item::new(
                item_id,
                item_name,
                item_notes,
                item_quantity,
                item_deleted,
                item_version,
                item_time_to_prepare,
            )),
            _ => Err(InsertError::Unknown),
        }
    }

    fn fetch_all(
        &self,
        table_id: TableId<IdType>,
        include_deleted: bool,
    ) -> Result<Vec<Item>, FetchAllError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchAllError::Unknown),
        };

        let item_rows = match Self::fetch_item_rows(&lock, IdType::from(table_id), None, include_deleted) {
            Ok(item_rows) => item_rows,
            _ => return Err(FetchAllError::Unknown),
        };

        let mut items = vec![];

        for item_row in item_rows {
            let item = match (
                ItemId::try_from(item_row.0.to_string()),
                ItemName::try_from(item_row.2),
                ItemNotes::try_from(item_row.3),
                ItemQuantity::try_from(item_row.4.to_string()),
                item_row.5,
                ItemVersion::try_from(item_row.6.to_string()),
                item_row.7,
            ) {
                (
                    Ok(id),
                    Ok(name),
                    Ok(notes),
                    Ok(quantity),
                    deleted,
                    Ok(version),
                    time_to_prepare,
                ) => Item::new(
                    id,
                    name,
                    notes,
                    quantity,
                    deleted,
                    version,
                    time_to_prepare,
                ),
                _ => return Err(FetchAllError::Unknown),
            };

            items.push(item);
        }

        Ok(items)
    }

    fn fetch_one(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
    ) -> Result<Item, FetchOneError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchOneError::Unknown),
        };

        let mut item_rows =
            match Self::fetch_item_rows(&lock, IdType::from(table_id), Some(IdType::from(item_id)), false)
            {
                Ok(rows) => rows,
                _ => return Err(FetchOneError::Unknown),
            };
        
        if item_rows.is_empty() {
            return Err(FetchOneError::UnknownItemId);
        }

        let item_row = item_rows.remove(0);

        match (
            ItemId::try_from(item_row.0.to_string()),
            ItemName::try_from(item_row.2),
            ItemNotes::try_from(item_row.3),
            ItemQuantity::try_from(item_row.4.to_string()),
            item_row.5,
            ItemVersion::try_from(item_row.6.to_string()),
            item_row.7,
        ) {
            (Ok(id), Ok(name), Ok(notes), Ok(quantity), deleted, Ok(version), time_to_prepare) => {
                Ok(Item::new(
                    id,
                    name,
                    notes,
                    quantity,
                    deleted,
                    version,
                    time_to_prepare,
                ))
            }
            _ => return Err(FetchOneError::Unknown)
        }
    }

    fn delete(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
    ) -> Result<(), DeleteError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(DeleteError::Unknown),
        };

        match lock.execute(
            "update item set deleted=1 where table_id=? and item_id = ? and deleted=0",
            params![IdType::from(table_id), IdType::from(item_id)],
        ) {
            Ok(0) => Err(DeleteError::UnknownItemId),
            Ok(_) => Ok(()),
            Err(SqliteFailure(_, Some(message))) => {
                println!("Message delete {}", message);
                return Err(DeleteError::Unknown);
            }
            _ => Err(DeleteError::Unknown),
        }
    }

    fn update(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
        item_name: ItemName,
        item_notes: ItemNotes,
        item_quantity: ItemQuantity<QuantityType>,
        item_deleted: bool,
        item_version: ItemVersion<VersionType>,
        item_time_to_prepare: String,
    ) -> Result<Item, UpdateError> {
        let mut lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(UpdateError::Unknown),
        };

        let item_rows =
            match Self::fetch_item_rows(&lock, IdType::from(table_id), Some(IdType::from(item_id)), false)
            {
                Ok(rows) => rows,
                _ => return Err(UpdateError::UnknownItemId),
            };
        
        if item_rows.is_empty() {
            return Err(UpdateError::UnknownItemId);
        }

        let row = &item_rows[0];

        if ItemVersion::from_int(row.6) > item_version {
            return Err(UpdateError::VersionConflict);
        }

        let new_version = ItemVersion::from_int(row.6) + ItemVersion::from_int(1);
        let transaction = match lock.transaction() {
            Ok(transaction) => transaction,
            _ => return Err(UpdateError::Unknown),
        };

        match transaction.execute(
            "update item set name = ?, notes = ?, quantity = ?, version = ?, time_to_prepare = ? where table_id = ? and item_id = ? and deleted=0",
            params![
                String::from(item_name.clone()),
                String::from(item_notes.clone()), 
                QuantityType::from(item_quantity.clone()), 
                VersionType::from(new_version.clone()),
                item_time_to_prepare,
                IdType::from(table_id),
                IdType::from(item_id)],
        ) {
            Ok(_) => {}
            Err(SqliteFailure(_, Some(message))) => {
                if message.contains("UNIQUE constraint failed") {
                    return Err(UpdateError::VersionConflict);
                } else {
                    println!("Message update {}", message);
                    return Err(UpdateError::Unknown);
                }
            }
            _ => return Err(UpdateError::Unknown),
        };

        match transaction.commit() {
            Ok(_) => Ok(Item::new(
                item_id,
                item_name,
                item_notes,
                item_quantity,
                item_deleted,
                new_version,
                item_time_to_prepare,
            )),
            _ => Err(UpdateError::Unknown),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use crate::domain::types::{ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, TableId};

    #[tokio::test]
    async fn it_should_create_db_with_table() {
        match SqliteRepository::try_new("") {
            Ok(_) => {},
            _ => panic!("Error while creating sqlite repo"),
        };
    }

    #[tokio::test]
    async fn it_should_insert_one_record() {
        let repo = match SqliteRepository::try_new("") {
            Ok(r) => Arc::new(r),
            _ => panic!("Error while creating sqlite repo"),
        };

        match repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }
    }

    #[tokio::test]
    async fn it_should_read_one_record() {
        let repo = match SqliteRepository::try_new("") {
            Ok(r) => Arc::new(r),
            _ => panic!("Error while creating sqlite repo"),
        };

        match repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }

        match repo.fetch_one(
            TableId::from_int(1),
            ItemId::from_int(1),
        ) {
            Ok(item) => {
                assert_eq!(item.id, ItemId::from_int(1));
                assert_eq!(item.name, ItemName::pizza());
                assert_eq!(item.notes, ItemNotes::some_notes());
                assert_eq!(item.quantity, ItemQuantity::one());
                assert_eq!(item.deleted, false);
                assert_eq!(item.version, ItemVersion::ver_one());
                assert_eq!(item.time_to_prepare, "2023/12/12".to_string())
            },
            _ => unreachable!()
        }
    }

    #[tokio::test]
    async fn it_should_update_one_record() {
        let repo = match SqliteRepository::try_new("") {
            Ok(r) => Arc::new(r),
            _ => panic!("Error while creating sqlite repo"),
        };

        match repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }

        match repo.update(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pasta(),
            ItemNotes::other_notes(),
            ItemQuantity::two(),
            false,
            ItemVersion::ver_one(),
            "2023/12/14".to_string()
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }

        match repo.fetch_one(
            TableId::from_int(1),
            ItemId::from_int(1),
        ) {
            Ok(item) => {
                assert_eq!(item.id, ItemId::from_int(1));
                assert_eq!(item.name, ItemName::pasta());
                assert_eq!(item.notes, ItemNotes::other_notes());
                assert_eq!(item.quantity, ItemQuantity::two());
                assert_eq!(item.deleted, false);
                assert_eq!(item.version, ItemVersion::from_int(2));
                assert_eq!(item.time_to_prepare, "2023/12/14".to_string())
            },
            _ => unreachable!()
        }
    }

    #[tokio::test]
    async fn it_should_delete_one_record() {
        let repo = match SqliteRepository::try_new("") {
            Ok(r) => Arc::new(r),
            _ => panic!("Error while creating sqlite repo"),
        };

        match repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }

        match repo.delete(
            TableId::from_int(1),
            ItemId::from_int(1),
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }

        match repo.fetch_one(
            TableId::from_int(1),
            ItemId::from_int(1),
        ) {
            Err(e) => {
                assert!(e == FetchOneError::UnknownItemId);
            },
            _ => unreachable!()
        }
    }

    #[tokio::test]
    async fn it_should_fetch_all_record() {
        let repo = match SqliteRepository::try_new("") {
            Ok(r) => Arc::new(r),
            _ => panic!("Error while creating sqlite repo"),
        };
        let ids: Vec<u32> = Vec::from([1, 2, 3]);
        ids.iter().for_each(|id| {
            repo.insert(
                TableId::from_int(1),
                ItemId::from_int(id.clone()),
                ItemName::pizza(),
                ItemNotes::some_notes(),
                ItemQuantity::one(),
                false,
                ItemVersion::ver_one(),
                "2023/12/12".to_string(),
            )
            .ok();
        });

        match repo.fetch_all(
            TableId::from_int(1), false
        ) {
            Ok(items) => {
                assert_eq!(ids.len(), items.len())
            }
            _ => unreachable!()
        }
        
        match repo.delete(
            TableId::from_int(1),
            ItemId::from_int(1),
        ) {
            Ok(_) => {},
            _ => unreachable!()
        }

        match repo.fetch_all(
            TableId::from_int(1), false
        ) {
            Ok(items) => {
                assert_eq!(ids.len() - 1, items.len())
            }
            _ => unreachable!()
        }
    }
}
