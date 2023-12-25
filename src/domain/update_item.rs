use std::sync::Arc;

use crate::domain::types::{IdType, Item, TableId};
use crate::repository::{Repository, UpdateError};

// Here can be found request and response structs and function execute() to 
// perform Repository update()


pub struct CreateOrUpdateRequest {
    pub table_id: TableId<IdType>,
    pub item: Item,
}

pub struct CreateOrUpdateResponse {
    pub item: Item,
}

pub enum Error {
    Unknown,
    UnknowTableId,
    UnknownItemId,
    VersionConflict,
}

pub fn execute(
    repo: Arc<dyn Repository>,
    req: CreateOrUpdateRequest,
) -> Result<CreateOrUpdateResponse, Error> {
    let mut cloned_it = req.item.clone();
    cloned_it.gen_time_to_prepare();
    match repo.update(
        req.table_id,
        cloned_it.id,
        cloned_it.name,
        cloned_it.notes,
        cloned_it.quantity,
        cloned_it.deleted,
        cloned_it.version,
        cloned_it.time_to_prepare,
    ) {
        Ok(item) => Ok(CreateOrUpdateResponse { item }),
        Err(UpdateError::Unknown) => Err(Error::Unknown),
        Err(UpdateError::UnknownItemId) => Err(Error::UnknownItemId),
        Err(UpdateError::UnknownTableId) => Err(Error::UnknowTableId),
        Err(UpdateError::VersionConflict) => Err(Error::VersionConflict),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::types::{
            ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, QuantityType, VersionType,
        },
        repository::inmemory::InMemoryRepository,
    };

    #[test]
    fn it_should_return_an_ok_when_request_is_valid() {
        let repo = Arc::new(InMemoryRepository::new());
        let same_table_id = 1;
        let same_item_id = 1;
        repo.insert(
            TableId::from_int(same_table_id),
            ItemId::from_int(same_item_id),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::from_int(1),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = CreateOrUpdateRequest::new(
            TableId::from_int(same_table_id),
            ItemId::from_int(same_item_id),
            ItemName::from_str("New item name".to_string()),
            ItemNotes::from_str("New notes".to_string()),
            ItemQuantity::from_int(2),
            false,
            ItemVersion::from_int(2),
            "".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Ok(res) => {
                assert_eq!(res.item.id, ItemId::from_int(1));
                assert_eq!(
                    res.item.name,
                    ItemName::from_str("New item name".to_string())
                );
                assert_eq!(res.item.notes, ItemNotes::from_str("New notes".to_string()));
                assert_eq!(res.item.quantity, ItemQuantity::from_int(2));
                assert_eq!(res.item.deleted, false);
                assert_eq!(res.item.version, ItemVersion::from_int(2));
            }
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_a_conflict_error_when_item_version_is_lower_than_in_storage() {
        let repo = Arc::new(InMemoryRepository::new());
        let same_table_id = 1;
        let same_item_id = 1;
        repo.insert(
            TableId::from_int(same_table_id),
            ItemId::from_int(same_item_id),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::from_int(2),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = CreateOrUpdateRequest::new(
            TableId::from_int(same_table_id),
            ItemId::from_int(same_item_id),
            ItemName::from_str("New item name".to_string()),
            ItemNotes::from_str("New notes".to_string()),
            ItemQuantity::from_int(2),
            false,
            ItemVersion::from_int(1),
            "".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Err(Error::VersionConflict) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_unknown_error_when_table_id_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = CreateOrUpdateRequest::new(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Err(Error::UnknowTableId) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_unknown_error_when_item_id_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = CreateOrUpdateRequest::new(
            TableId::from_int(1),
            ItemId::from_int(2),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Err(Error::UnknownItemId) => {}
            _ => unreachable!(),
        };
    }

    impl CreateOrUpdateRequest {
        fn new(
            table_id: TableId<IdType>,
            item_id: ItemId<IdType>,
            item_name: ItemName,
            item_notes: ItemNotes,
            item_quantity: ItemQuantity<QuantityType>,
            item_deleted: bool,
            item_version: ItemVersion<VersionType>,
            item_time_to_prepare: String,
        ) -> Self {
            Self {
                table_id: table_id,
                item: Item {
                    id: item_id,
                    name: item_name,
                    notes: item_notes,
                    quantity: item_quantity,
                    deleted: item_deleted,
                    version: item_version,
                    time_to_prepare: item_time_to_prepare,
                },
            }
        }
    }
}
