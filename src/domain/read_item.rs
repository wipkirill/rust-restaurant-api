use std::sync::Arc;

use crate::domain::types::{IdType, Item, TableId};
use crate::repository::{FetchOneError, Repository};

use super::types::ItemId;

// Here can be found request and response structs and function execute() to 
// perform Repository call fetch_one()


pub struct ReadRequest {
    pub table_id: TableId<IdType>,
    pub item_id: ItemId<IdType>,
}

pub struct ReadResponse {
    pub item: Item,
}

pub enum Error {
    Unknown,
    UnknowTableId,
    UnknownItemId,
}

pub fn execute(repo: Arc<dyn Repository>, req: ReadRequest) -> Result<ReadResponse, Error> {
    match repo.fetch_one(req.table_id, req.item_id) {
        Ok(item) => Ok(ReadResponse { item }),
        Err(FetchOneError::UnknownItemId) => Err(Error::UnknownItemId),
        Err(FetchOneError::UnknownTableId) => Err(Error::UnknowTableId),
        Err(FetchOneError::Unknown) => Err(Error::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::types::{ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion},
        repository::inmemory::InMemoryRepository,
    };

    #[test]
    fn it_should_return_an_ok_when_request_is_valid() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::from_int(1),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = ReadRequest::new(TableId::from_int(1), ItemId::from_int(1));

        let res = execute(repo, req);

        match res {
            Ok(res) => {
                assert_eq!(res.item.id, ItemId::from_int(1));
                assert_eq!(res.item.name, ItemName::pizza());
                assert_eq!(res.item.notes, ItemNotes::some_notes());
                assert_eq!(res.item.quantity, ItemQuantity::one());
                assert_eq!(res.item.deleted, false);
                assert_eq!(res.item.version, ItemVersion::from_int(1));
            }
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_unknown_table_id_error_when_table_id_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = ReadRequest::new(TableId::from_int(1), ItemId::from_int(1));
        let res = execute(repo, req);

        match res {
            Err(Error::UnknowTableId) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_unknown_item_id_error_when_item_id_not_found() {
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

        let req = ReadRequest::new(TableId::from_int(1), ItemId::from_int(2));
        let res = execute(repo, req);

        match res {
            Err(Error::UnknownItemId) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_not_found_when_item_deleted() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            TableId::from_int(1),
            ItemId::from_int(1),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            true,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = ReadRequest::new(TableId::from_int(1), ItemId::from_int(1));
        let res = execute(repo, req);

        match res {
            Err(Error::UnknownItemId) => {}
            _ => unreachable!(),
        };
    }

    impl ReadRequest {
        fn new(table_id: TableId<IdType>, item_id: ItemId<IdType>) -> Self {
            Self { table_id, item_id }
        }
    }
}
