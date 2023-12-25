use std::sync::Arc;

use crate::domain::types::{IdType, Item, TableId};
use crate::repository::{FetchAllError, Repository};

// Here can be found request and response structs and function execute() to 
// perform Repository call fetch_all()

pub struct ReadAllRequest {
    pub table_id: TableId<IdType>,
    pub include_deleted: bool,
    pub filter: String,
    pub sort_by: String,
}

pub struct ReadAllResponse {
    pub items: Vec<Item>,
}

pub enum Error {
    Unknown,
    UnknowTableId,
}

pub fn execute(repo: Arc<dyn Repository>, req: ReadAllRequest) -> Result<ReadAllResponse, Error> {
    match repo.fetch_all(req.table_id, req.include_deleted) {
        Ok(items) => Ok(ReadAllResponse { items }),
        Err(FetchAllError::UnknownTableId) => Err(Error::UnknowTableId),
        Err(FetchAllError::Unknown) => Err(Error::Unknown),
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

        let req = ReadAllRequest::new(
            TableId::from_int(1),
            false,
            String::from(""),
            String::from(""),
        );

        let res = execute(repo, req);

        match res {
            Ok(res) => {
                assert!(res.items.len() == 1);
                assert_eq!(res.items[0].id, ItemId::from_int(1));
                assert_eq!(res.items[0].name, ItemName::pizza());
                assert_eq!(res.items[0].notes, ItemNotes::some_notes());
                assert_eq!(res.items[0].quantity, ItemQuantity::one());
                assert_eq!(res.items[0].deleted, false);
                assert_eq!(res.items[0].version, ItemVersion::from_int(1));
            }
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_an_unknown_table_id_error_when_table_id_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = ReadAllRequest::new(
            TableId::from_int(1),
            false,
            String::from(""),
            String::from(""),
        );
        let res = execute(repo, req);

        match res {
            Err(Error::UnknowTableId) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_only_not_deleted_items_when_not_include_deleted() {
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
        repo.insert(
            TableId::from_int(1),
            ItemId::from_int(2),
            ItemName::pizza(),
            ItemNotes::some_notes(),
            ItemQuantity::one(),
            true,
            ItemVersion::ver_one(),
            "2023/12/12".to_string(),
        )
        .ok();

        let req = ReadAllRequest::new(
            TableId::from_int(1),
            false,
            String::from(""),
            String::from(""),
        );
        let res = execute(repo, req);

        match res {
            Ok(res) => {
                assert!(res.items.len() == 1);
                assert_eq!(res.items[0].id, ItemId::from_int(1));
                assert_eq!(res.items[0].name, ItemName::pizza());
                assert_eq!(res.items[0].notes, ItemNotes::some_notes());
                assert_eq!(res.items[0].quantity, ItemQuantity::one());
                assert_eq!(res.items[0].deleted, false);
                assert_eq!(res.items[0].version, ItemVersion::from_int(1));
            }
            _ => unreachable!(),
        };
    }

    impl ReadAllRequest {
        fn new(
            table_id: TableId<IdType>,
            include_deleted: bool,
            filter: String,
            sort_by: String,
        ) -> Self {
            Self {
                table_id,
                include_deleted,
                filter,
                sort_by,
            }
        }
    }
}
