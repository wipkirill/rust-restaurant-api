use crate::api::helpers::{parse_numeric_id, to_json, FailMsg, OpItemsResponse};
use crate::api::HttpStatus;
use crate::api::{Request, Response};
use crate::domain::read_item::{execute, Error, ReadRequest};
use crate::domain::read_items::{execute as execute_fetch_all, Error as ErrorAll, ReadAllRequest};
use crate::domain::types::{IdType, ItemId, TableId};
use crate::repository::Repository;
use std::collections::HashMap;
use std::sync::Arc;

// This file contains functions to handle GET requests

pub async fn read_item_handler(request: Request) -> Response {
    // Check table exists
    let url_pattern = "/tables/:tid/items/:id";
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

    //retrieve item
    match execute(repo, ReadRequest { table_id, item_id }) {
        Ok(res) => return to_json(res.item, HttpStatus::OK).await,
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

pub async fn read_items_handler(request: Request) -> Response {
    // Check table exists
    let url_pattern = "/tables/:tid/items";
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

    let mut op_status: OpItemsResponse = HashMap::new();
    //retrieve items
    match execute_fetch_all(
        repo,
        ReadAllRequest {
            table_id: table_id,
            include_deleted: false,
            filter: String::from(""),
            sort_by: String::from(""),
        },
    ) {
        Ok(res) => {
            for r in &res.items {
                op_status.insert(r.id, serde_json::to_value(r).unwrap());
            }
            return to_json(op_status, HttpStatus::OK).await;
        }
        Err(ErrorAll::UnknowTableId) => {
            return to_json(
                FailMsg {
                    msg: "Unknown table id".to_string(),
                },
                HttpStatus::BAD_REQUEST,
            )
            .await
        }
        Err(ErrorAll::Unknown) => {
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

#[cfg(test)]
mod test {
    use crate::api::HttpStatus;
    use crate::domain::types::{ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, TableId};
    use crate::repository::inmemory::InMemoryRepository;
    use crate::repository::Repository;
    use futures::future::join_all;
    use hyper::http;
    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use std::sync::Arc;

    use crate::handle;

    #[tokio::test]
    async fn it_should_return_ok_read_one_item() {
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
            .method(http::Method::GET)
            .uri("/tables/1/items/1")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(result.as_str()).unwrap();
        assert_eq!(json["name"], String::from(ItemName::pizza()));
        assert_eq!(json["notes"], String::from(ItemNotes::some_notes()));
        assert_eq!(json["quantity"], 1);
        assert_eq!(json["deleted"], false);
        assert!(json["time_to_prepare"] != "");
        assert_eq!(json["version"], 1);
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
            .method(http::Method::GET)
            .uri("/tables/1/items/2")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context.clone());
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::NOT_FOUND);

        body = "";
        request = hyper::Request::builder()
            .method(http::Method::GET)
            .uri("/tables/2/items/2")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context.clone());
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
    }

    #[tokio::test]
    async fn it_should_return_ok_read_multi_items() {
        let repo = InMemoryRepository::new();
        let table_ids = vec![1, 2];
        let item_ids = vec![1, 2];
        table_ids.iter().for_each(|table_id: &u32| {
            item_ids.iter().for_each(|id: &u32| {
                repo.insert(
                    TableId::from_int(table_id.clone()),
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
        });
        let context: Arc<dyn Repository> = Arc::new(repo);

        join_all(table_ids.iter().map(|table_id| async {
            let body: &str = "";
            let mut request = hyper::Request::builder()
                .method(http::Method::GET)
                .uri(format!("/tables/{}/items", table_id.to_string()))
                .body(hyper::Body::from(body))
                .unwrap();
            request.extensions_mut().insert(context.clone());
            let response = handle(request).await;

            assert_eq!(response.status(), HttpStatus::OK);

            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let result = String::from_utf8(body.to_vec()).unwrap();
            let json: Value = serde_json::from_str(result.as_str()).unwrap();
            item_ids.iter().for_each(|id| {
                let obj = &json[id.to_string()];
                assert_eq!(obj["name"], String::from(ItemName::pizza()));
                assert_eq!(obj["notes"], String::from(ItemNotes::some_notes()));
                assert_eq!(obj["quantity"], 1);
                assert_eq!(obj["deleted"], false);
                assert!(obj["time_to_prepare"] != "");
                assert_eq!(obj["version"], 1);
            });
        }))
        .await;
    }
}
