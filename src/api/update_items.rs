use crate::api::helpers::{
    extract_string_payload, json_body, parse_create_or_update_items, parse_numeric_id, to_json,
    FailMsg, OpStatusResponse,
};
use crate::api::HttpStatus;
use crate::api::{Request, Response};
use crate::domain::types::{IdType, Item, TableId};
use crate::domain::update_item::{execute, CreateOrUpdateRequest, Error};
use crate::repository::Repository;
use std::collections::HashMap;
use std::sync::Arc;

use super::helpers::StatusWithBody;

// This file contains functions to handle PUT requests

pub async fn update_items_handler(request: Request) -> Response {
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

    // parse item(s)
    let payload: String = extract_string_payload(request)
        .await
        .unwrap_or_else(|_| String::from(""));
    let updated_items: Vec<Item> = match parse_create_or_update_items(&payload) {
        Ok(items) => items,
        Err(err) => {
            return to_json(
                FailMsg {
                    msg: err.to_string(),
                },
                HttpStatus::OK,
            )
            .await
        }
    };

    // update item(s)
    let mut op_status: OpStatusResponse = HashMap::new();
    updated_items.iter().for_each(|u_item| {
        match execute(
            repo.clone(),
            CreateOrUpdateRequest {
                table_id: table_id,
                item: u_item.clone(),
            },
        ) {
            Ok(res) => op_status.insert(
                res.item.id,
                StatusWithBody {
                    status: HttpStatus::OK.as_u16(),
                    body: json_body::<Item>(res.item.clone()),
                },
            ),
            Err(Error::Unknown) => op_status.insert(
                u_item.id,
                StatusWithBody {
                    status: HttpStatus::BAD_REQUEST.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Server error".to_string(),
                    }),
                },
            ),
            Err(Error::UnknowTableId) => op_status.insert(
                u_item.id,
                StatusWithBody {
                    status: HttpStatus::NOT_FOUND.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Unknown table id".to_string(),
                    }),
                },
            ),
            Err(Error::UnknownItemId) => op_status.insert(
                u_item.id,
                StatusWithBody {
                    status: HttpStatus::NOT_FOUND.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Unknown item id".to_string(),
                    }),
                },
            ),
            Err(Error::VersionConflict) => op_status.insert(
                u_item.id,
                StatusWithBody {
                    status: HttpStatus::BAD_REQUEST.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Version mismatch: server has newer verstion".to_string(),
                    }),
                },
            ),
        };
    });

    match op_status.len() {
        1 => {
            let response = op_status.get(&updated_items[0].id).unwrap();
            let status = HttpStatus::from_u16(response.status).unwrap();
            to_json(&response.body, status).await
        }
        _ => to_json(op_status, HttpStatus::MULTI_STATUS).await,
    }
}

#[cfg(test)]
mod test {
    use hyper::http;

    use crate::api::HttpStatus;
    use crate::domain::types::{ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, TableId};
    use crate::repository::inmemory::InMemoryRepository;
    use crate::repository::Repository;
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};
    use std::sync::Arc;

    use crate::handle;

    #[tokio::test]
    async fn it_should_return_ok_update_one_item() {
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

        let body = r#"
        {
            "1": {
                "name": "Name from menu1",
                "notes": "Notes from waiter1",
                "quantity": 2
            }
        }"#;
        let mut request = hyper::Request::builder()
            .method(hyper::http::Method::PUT)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(result.as_str()).unwrap();
        assert_eq!(json["name"], "Name from menu1");
        assert_eq!(json["notes"], "Notes from waiter1");
        assert_eq!(json["quantity"], 2);
        assert_eq!(json["deleted"], false);
        assert!(json["time_to_prepare"] != "");
        assert_eq!(json["version"], 2);
    }

    #[tokio::test]
    async fn it_should_return_ok_update_multi_item() {
        let repo: InMemoryRepository = InMemoryRepository::new();
        let ids: Vec<u32> = Vec::from([1, 2, 3]);
        ids.iter().for_each(|id| {
            repo.insert(
                TableId::from_int(1),
                ItemId::from_int(id.clone()),
                ItemName::from_str(format!("Name from menu {}", id)),
                ItemNotes::from_str(format!("Notes from waiter {}", id)),
                ItemQuantity::one(),
                false,
                ItemVersion::ver_one(),
                format!("2023/12/12-{}", id),
            )
            .ok();
        });
        let context: Arc<dyn Repository> = Arc::new(repo);

        let mut body_json = json!({});
        ids.iter().for_each(|id| {
            body_json[id.to_string()] = json!({
                "name": format!("Name from menu {} updated", id),
                "notes": format!("Notes from waiter {} updated", id),
                "quantity": 2
            });
        });
        let mut request = hyper::Request::builder()
            .method(http::Method::PUT)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body_json.to_string()))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::MULTI_STATUS);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(result.as_str()).unwrap();

        ids.iter().for_each(|id| {
            //println!("{}", json[id.to_string()]["status"].to_string());
            assert_eq!(json[id.to_string()]["status"], HttpStatus::OK.as_u16());
            let obj = &json[id.to_string()]["body"];
            assert_eq!(obj["name"], format!("Name from menu {} updated", id));
            assert_eq!(obj["notes"], format!("Notes from waiter {} updated", id));
            assert_eq!(obj["quantity"], 2);
            assert_eq!(obj["deleted"], false);
            assert!(obj["time_to_prepare"] != "");
            assert_eq!(obj["version"], 2);
        });
    }

    #[tokio::test]
    async fn it_should_fail_update_item_not_found() {
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

        let body = r#"
        {
            "2": {
                "name": "Name from menu1",
                "notes": "Notes from waiter1",
                "quantity": 2
            }
        }"#;
        let mut request = hyper::Request::builder()
            .method(http::Method::PUT)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
    }
}
