use crate::api::helpers::{
    extract_string_payload, json_body, parse_delete_items_request, parse_numeric_id, to_json,
    FailMsg, OpStatusResponse, StatusWithBody,
};
use crate::api::HttpStatus;
use crate::api::{Request, Response};
use crate::domain::delete_item::{execute, DeleteOneRequest, DeleteOneResponse, Error};
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::types::{IdType, ItemId, TableId};
use crate::repository::Repository;

// This function is used to handle deletion of single item
pub async fn delete_item_handler(request: Request) -> Response {
    // Check table exists
    let url_pattern = "/tables/:tid/items/:id";
    // Check table exists
    let table_id =
        match parse_numeric_id::<TableId<IdType>>(&request.uri().path(), url_pattern, "tid") {
            Ok(tid) => tid,
            Err(err) => {
                return to_json(
                    FailMsg {
                        msg: err.to_string(),
                    },
                    HttpStatus::BAD_REQUEST,
                )
                .await
            }
        };
    let repo: Arc<dyn Repository> = Arc::clone(request.extensions().get().unwrap());
    // Read ItemId
    let item_id = match parse_numeric_id::<ItemId<IdType>>(&request.uri().path(), url_pattern, "id")
    {
        Ok(it_id) => it_id,
        Err(err) => {
            return to_json(
                FailMsg {
                    msg: err.to_string(),
                },
                HttpStatus::BAD_REQUEST,
            )
            .await
        }
    };

    match execute(repo, DeleteOneRequest { table_id, item_id }) {
        Ok(res) => return to_json(res, HttpStatus::OK).await,
        Err(Error::UnknowTableId) => {
            return to_json(
                FailMsg {
                    msg: "Unknown table id".to_string(),
                },
                HttpStatus::NOT_FOUND,
            )
            .await
        }
        Err(Error::UnknownItemId) => {
            return to_json(
                FailMsg {
                    msg: "Unknown item id".to_string(),
                },
                HttpStatus::NOT_FOUND,
            )
            .await
        }
        Err(Error::Unknown) => {
            return to_json(
                FailMsg {
                    msg: "Server error".to_string(),
                },
                HttpStatus::BAD_REQUEST,
            )
            .await
        }
    }
}

// This function is used to handle deletion of multiple items
pub async fn delete_items_handler(request: Request) -> Response {
    // Check table exists
    let url_pattern = "/tables/:tid/items";
    // Check table exists
    let table_id =
        match parse_numeric_id::<TableId<IdType>>(&request.uri().path(), url_pattern, "tid") {
            Ok(tid) => tid,
            Err(err) => {
                return to_json(
                    FailMsg {
                        msg: err.to_string(),
                    },
                    HttpStatus::BAD_REQUEST,
                )
                .await
            }
        };
    let repo: Arc<dyn Repository> = Arc::clone(request.extensions().get().unwrap());

    // parse item ids
    let payload: String = extract_string_payload(request)
        .await
        .unwrap_or_else(|_| String::from(""));
    let items_to_delete = match parse_delete_items_request(&payload) {
        Ok(items) => items,
        Err(err) => {
            return to_json(
                FailMsg {
                    msg: err.to_string(),
                },
                HttpStatus::BAD_REQUEST,
            )
            .await
        }
    };

    let mut op_status: OpStatusResponse = HashMap::new();
    items_to_delete.into_iter().for_each(|id| {
        match execute(
            repo.clone(),
            DeleteOneRequest {
                table_id: table_id,
                item_id: id,
            },
        ) {
            Ok(_) => op_status.insert(
                id,
                StatusWithBody {
                    status: HttpStatus::OK.as_u16(),
                    body: json_body::<DeleteOneResponse>(DeleteOneResponse {}),
                },
            ),
            Err(Error::UnknownItemId) => op_status.insert(
                id,
                StatusWithBody {
                    status: HttpStatus::NOT_FOUND.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Unknow item id".to_string(),
                    }),
                },
            ),
            Err(Error::Unknown) => op_status.insert(
                id,
                StatusWithBody {
                    status: HttpStatus::BAD_REQUEST.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Server error".to_string(),
                    }),
                },
            ),
            Err(Error::UnknowTableId) => op_status.insert(
                id,
                StatusWithBody {
                    status: HttpStatus::NOT_FOUND.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Unknown table id".to_string(),
                    }),
                },
            ),
        };
    });
    to_json(op_status, HttpStatus::MULTI_STATUS).await
}

#[cfg(test)]
mod test {
    use crate::api::HttpStatus;
    use crate::domain::types::{ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, TableId};
    use crate::repository::inmemory::InMemoryRepository;
    use crate::repository::Repository;
    use hyper::http;

    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use std::sync::Arc;

    use crate::handle;

    #[tokio::test]
    async fn it_should_return_ok_delete_one_item() {
        let repo: InMemoryRepository = InMemoryRepository::new();
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
        let context: Arc<dyn Repository> = Arc::new(repo);

        let body = "";
        let mut request = hyper::Request::builder()
            .method(http::Method::DELETE)
            .uri("/tables/1/items/1")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context.clone());
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::OK);

        match context.fetch_one(TableId::from_int(1), ItemId::from_int(1)) {
            Err(_) => {}
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn it_should_fail_item_or_table_not_found() {
        let repo: InMemoryRepository = InMemoryRepository::new();
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
        let context: Arc<dyn Repository> = Arc::new(repo);

        let mut body = "";
        let mut request = hyper::Request::builder()
            .method(http::Method::DELETE)
            .uri("/tables/1/items/2")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context.clone());
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::NOT_FOUND);

        body = "";
        request = hyper::Request::builder()
            .method(http::Method::DELETE)
            .uri("/tables/2/items/2")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context.clone());
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
    }

    #[tokio::test]
    async fn it_should_return_ok_delete_multi_items() {
        let repo: InMemoryRepository = InMemoryRepository::new();
        let mut ids: Vec<u32> = Vec::from([1, 2]);
        ids.iter_mut().for_each(|id| {
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
        let context: Arc<dyn Repository> = Arc::new(repo);

        let body = r#"
        {
            "ids": [1, 2, 3]
        }"#;
        let mut request = hyper::Request::builder()
            .method(http::Method::DELETE)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context.clone());
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::MULTI_STATUS);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(result.as_str()).unwrap();
        ids.iter().for_each(|id| {
            assert_eq!(json[&id.to_string()]["status"], HttpStatus::OK.as_u16());
            match context.fetch_one(TableId::from_int(1), ItemId::from_int(id.clone())) {
                Err(_) => {}
                _ => unreachable!(),
            }
        });
        assert_eq!(json["3"]["status"], HttpStatus::NOT_FOUND.as_u16());
    }
}
