pub mod inmemory;
pub mod sqlite;

// Repository interface and errors

use crate::domain::types::{
    IdType, Item, ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, QuantityType, TableId,
    VersionType,
};

pub enum InsertError {
    Conflict,
    Unknown,
}

pub enum UpdateError {
    UnknownItemId,
    UnknownTableId,
    Unknown,
    VersionConflict,
}

pub enum FetchAllError {
    Unknown,
    UnknownTableId,
}

#[derive(PartialEq)]
pub enum FetchOneError {
    Unknown,
    UnknownItemId,
    UnknownTableId,
}

pub enum DeleteError {
    Unknown,
    UnknownItemId,
    UnknownTableId,
}

pub trait Repository: Send + Sync {
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
    ) -> Result<Item, InsertError>;

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
    ) -> Result<Item, UpdateError>;

    fn fetch_all(
        &self,
        table_id: TableId<IdType>,
        _include_deleted: bool,
    ) -> Result<Vec<Item>, FetchAllError>;

    fn fetch_one(
        &self,
        table_id: TableId<IdType>,
        item_id: ItemId<IdType>,
    ) -> Result<Item, FetchOneError>;

    fn delete(&self, table_id: TableId<IdType>, item_id: ItemId<IdType>)
        -> Result<(), DeleteError>;
}
