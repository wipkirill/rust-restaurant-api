use crate::domain::types::{
    IdType, Item, ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, QuantityType, TableId,
    VersionType,
};
use crate::repository::*;
use std::collections::HashMap;
use std::sync::Mutex;

// In memory repository implementation based on HashMap

pub struct InMemoryRepository {
    error: bool,
    items: Mutex<HashMap<TableId<IdType>, Vec<Item>>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        let items: Mutex<HashMap<TableId<IdType>, Vec<Item>>> = Mutex::new(HashMap::new());
        Self {
            error: false,
            items,
        }
    }

    #[cfg(test)]
    pub fn with_error(self) -> Self {
        Self {
            error: true,
            ..self
        }
    }
}

impl Repository for InMemoryRepository {
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
        if self.error {
            return Err(InsertError::Unknown);
        }

        let mut lock = match self.items.lock() {
            Ok(lock) => lock,
            _ => return Err(InsertError::Unknown),
        };

        if !lock.contains_key(&table_id) {
            lock.insert(table_id, vec![]);
        }

        if lock[&table_id]
            .iter()
            .any(|item| item.id == item_id && item.deleted == false)
        {
            return Err(InsertError::Conflict);
        }

        let item = Item::new(
            item_id,
            item_name,
            item_notes,
            item_quantity,
            item_deleted,
            item_version,
            item_time_to_prepare,
        );
        lock.get_mut(&table_id).unwrap().push(item.clone());
        Ok(item)
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
        if self.error {
            return Err(UpdateError::Unknown);
        }

        let mut lock = match self.items.lock() {
            Ok(lock) => lock,
            _ => return Err(UpdateError::Unknown),
        };

        if !lock.contains_key(&table_id) {
            return Err(UpdateError::UnknownTableId);
        }

        let pos = match lock[&table_id]
            .iter()
            .position(|item| item.id == item_id && item.deleted == false)
        {
            Some(pos) => pos,
            None => return Err(UpdateError::UnknownItemId),
        };

        if lock.get_mut(&table_id).unwrap()[pos].version > item_version {
            return Err(UpdateError::VersionConflict);
        }

        let mut current_version = lock.get_mut(&table_id).unwrap()[pos].version.clone();

        current_version += ItemVersion::from_int(1);

        let item = Item::new(
            item_id,
            item_name,
            item_notes,
            item_quantity,
            item_deleted,
            current_version,
            item_time_to_prepare,
        );
        lock.get_mut(&table_id).unwrap()[pos] = item.clone();
        Ok(item)
    }

    fn fetch_all(
        &self,
        table_id: TableId<IdType>,
        _include_deleted: bool,
    ) -> Result<Vec<Item>, FetchAllError> {
        if self.error {
            return Err(FetchAllError::Unknown);
        }

        let lock = match self.items.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchAllError::Unknown),
        };

        if !lock.contains_key(&table_id) {
            return Err(FetchAllError::UnknownTableId);
        }

        let cond = match _include_deleted {
            true => |_: &&Item| true,
            false => |it: &&Item| it.deleted == false,
        };
        let mut items: Vec<_> = lock[&table_id].iter().filter(cond).cloned().collect();
        items.sort_by(|a, b| a.id.to_string().cmp(&b.id.to_string()));
        Ok(items)
    }

    fn fetch_one(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
    ) -> Result<Item, FetchOneError> {
        if self.error {
            return Err(FetchOneError::Unknown);
        }

        let lock = match self.items.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchOneError::Unknown),
        };

        if !lock.contains_key(&table_id) {
            return Err(FetchOneError::UnknownTableId);
        }

        match lock[&table_id]
            .iter()
            .find(|p| p.id == item_id && p.deleted == false)
        {
            Some(item) => Ok(item.clone()),
            None => Err(FetchOneError::UnknownItemId),
        }
    }

    fn delete(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
    ) -> Result<(), DeleteError> {
        if self.error {
            return Err(DeleteError::Unknown);
        }

        let mut lock = match self.items.lock() {
            Ok(lock) => lock,
            _ => return Err(DeleteError::Unknown),
        };

        if !lock.contains_key(&table_id) {
            return Err(DeleteError::UnknownTableId);
        }

        let index = match lock[&table_id]
            .iter()
            .position(|p| p.id == item_id && p.deleted == false)
        {
            Some(index) => index,
            None => return Err(DeleteError::UnknownItemId),
        };

        lock.get_mut(&table_id).unwrap()[index].deleted = true;
        Ok(())
    }
}
