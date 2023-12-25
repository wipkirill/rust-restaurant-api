use super::types::ItemId;
use crate::domain::types::{IdType, TableId};
use crate::repository::{DeleteError, Repository};
use serde::Serialize;
use std::sync::Arc;

pub struct DeleteOneRequest {
    pub table_id: TableId<IdType>,
    pub item_id: ItemId<IdType>,
}

// Here can be found request and response structs and function execute() to 
// perform Repository call delete()

#[derive(Serialize)]
pub struct DeleteOneResponse {}

pub enum Error {
    Unknown,
    UnknowTableId,
    UnknownItemId,
}

pub fn execute(
    repo: Arc<dyn Repository>,
    req: DeleteOneRequest,
) -> Result<DeleteOneResponse, Error> {
    match repo.delete(req.table_id, req.item_id) {
        Ok(_) => Ok(DeleteOneResponse {}),
        Err(DeleteError::UnknownItemId) => Err(Error::UnknownItemId),
        Err(DeleteError::UnknownTableId) => Err(Error::UnknowTableId),
        Err(DeleteError::Unknown) => Err(Error::Unknown),
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
        let table_id = TableId::from_int(1);
        let item_id = ItemId::from_int(1);
        repo.insert(
            table_id,
            item_id,
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            false,
            ItemVersion::from_int(1),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = DeleteOneRequest::new(table_id, item_id);

        let res = execute(repo, req);

        match res {
            Ok(_) => {}
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_unknown_table_id_error_when_table_id_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = DeleteOneRequest::new(TableId::from_int(1), ItemId::from_int(1));
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

        let req = DeleteOneRequest::new(TableId::from_int(1), ItemId::from_int(2));
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

        let req = DeleteOneRequest::new(TableId::from_int(1), ItemId::from_int(1));
        let res = execute(repo, req);

        match res {
            Err(Error::UnknownItemId) => {}
            _ => unreachable!(),
        };
    }

    impl DeleteOneRequest {
        fn new(table_id: TableId<IdType>, item_id: ItemId<IdType>) -> Self {
            Self { table_id, item_id }
        }
    }
}
