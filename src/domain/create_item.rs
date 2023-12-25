use std::sync::Arc;

use crate::domain::types::{IdType, Item, TableId};
use crate::repository::{InsertError, Repository};

// Here can be found request and response structs and function execute() to 
// perform Repository call insert()

pub struct CreateItemRequest {
    pub table_id: TableId<IdType>,
    pub item: Item,
}

pub struct CreateItemResponse {
    pub item: Item,
}

pub enum Error {
    Conflict,
    Unknown,
}

pub fn execute(
    repo: Arc<dyn Repository>,
    req: CreateItemRequest,
) -> Result<CreateItemResponse, Error> {
    let mut cloned_it = req.item.clone();
    cloned_it.gen_time_to_prepare();
    match repo.insert(
        req.table_id,
        cloned_it.id,
        cloned_it.name,
        cloned_it.notes,
        cloned_it.quantity,
        cloned_it.deleted,
        cloned_it.version,
        cloned_it.time_to_prepare,
    ) {
        Ok(item) => Ok(CreateItemResponse { item }),
        Err(InsertError::Conflict) => Err(Error::Conflict),
        Err(InsertError::Unknown) => Err(Error::Unknown),
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
        let req = CreateItemRequest::new(
            TableId::id_one(),
            ItemId::id_one(),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Ok(_) => {}
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_a_conflict_error_when_item_already_exists() {
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
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = CreateItemRequest::new(
            TableId::from_int(same_table_id),
            ItemId::from_int(same_item_id),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Err(Error::Conflict) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_should_return_ok_when_deleted_item_exists() {
        let repo = Arc::new(InMemoryRepository::new());
        let table_id = 1;
        let item_id = 1;
        repo.insert(
            TableId::from_int(table_id),
            ItemId::from_int(item_id),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            true,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = CreateItemRequest::new(
            TableId::from_int(table_id),
            ItemId::from_int(item_id),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        );

        let res = execute(repo, req);

        match res {
            Ok(_) => {}
            _ => unreachable!(),
        }
    }
    #[test]
    fn it_should_return_an_unknown_error_when_an_unexpected_error_happens() {
        let repo = Arc::new(InMemoryRepository::new().with_error());
        let req = CreateItemRequest::new(
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
            Err(Error::Unknown) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_item() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = CreateItemRequest::new(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::ver_one(),
            String::from("2023/12/12"),
        );

        let res = execute(repo, req);

        match res {
            Ok(res) => {
                assert_eq!(res.item.id, ItemId::from_int(1));
                assert_eq!(res.item.name, ItemName::pizza());
                assert_eq!(res.item.notes, ItemNotes::some_notes());
                assert_eq!(res.item.quantity, ItemQuantity::one());
                assert_eq!(res.item.deleted, false);
                assert_eq!(res.item.version, ItemVersion::ver_one());
            }
            _ => unreachable!(),
        };
    }

    impl CreateItemRequest {
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
