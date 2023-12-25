use crate::api::helpers::{
    extract_string_payload, json_body, parse_create_or_update_items, parse_numeric_id, to_json,
    FailMsg, OpStatusResponse, StatusWithBody,
};
use crate::api::HttpStatus;
use crate::api::{Request, Response};
use crate::domain::create_item::{execute, CreateItemRequest, Error};
use crate::domain::types::{IdType, Item, TableId};
use crate::repository::Repository;
use std::collections::HashMap;
use std::sync::Arc;

// This function handles POST requests to create 
pub async fn create_items_handler(request: Request) -> Response {
    // Read table id
    let table_id = match parse_numeric_id::<TableId<IdType>>(
        &request.uri().path(),
        "/tables/:tid/items",
        "tid",
    ) {
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
    let new_items = match parse_create_or_update_items(&payload) {
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

    // insert item(s) and make response for each 
    let mut op_status: OpStatusResponse = HashMap::new();
    new_items.iter().for_each(|item| {
        match execute(
            repo.clone(),
            CreateItemRequest {
                table_id: table_id,
                item: item.clone(),
            },
        ) {
            Ok(res) => op_status.insert(
                res.item.id,
                StatusWithBody {
                    status: HttpStatus::CREATED.as_u16(),
                    body: json_body::<Item>(res.item.clone()),
                },
            ),
            Err(Error::Conflict) => op_status.insert(
                item.id,
                StatusWithBody {
                    status: HttpStatus::BAD_REQUEST.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Item already exists".to_string(),
                    }),
                },
            ),
            Err(Error::Unknown) => op_status.insert(
                item.id,
                StatusWithBody {
                    status: HttpStatus::BAD_REQUEST.as_u16(),
                    body: json_body::<FailMsg>(FailMsg {
                        msg: "Server error".to_string(),
                    }),
                },
            ),
        };
    });

    match op_status.len() {
        1 => {
            let response = op_status.get(&new_items[0].id).unwrap();
            let status = HttpStatus::from_u16(response.status).unwrap();
            to_json(&response.body, status).await
        }
        _ => to_json(op_status, HttpStatus::MULTI_STATUS).await,
    }
}

#[cfg(test)]
mod test {
    use crate::api::HttpStatus;
    use crate::repository::inmemory::InMemoryRepository;
    use crate::repository::Repository;
    use futures::future::join_all;
    use hyper::http;
    use pretty_assertions::assert_eq;
    use select::document::Document;
    use select::predicate::Name;
    use serde_json::Value;
    use std::sync::Arc;

    use crate::handle;

    #[tokio::test]
    async fn it_should_return_ok_create_one_item() {
        let repo: InMemoryRepository = InMemoryRepository::new();
        let context: Arc<dyn Repository> = Arc::new(repo);
        let body = r#"
        {
            "1": {
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": 1
            }
        }"#;
        let mut request = hyper::Request::builder()
            .method(http::Method::POST)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(result.as_str()).unwrap();
        assert_eq!(json["name"], "Name from menu");
        assert_eq!(json["notes"], "Notes from waiter");
        assert_eq!(json["quantity"], 1);
        assert_eq!(json["version"], 1);
        assert_eq!(json["deleted"], false);
        assert!(json["time_to_prepare"] != "");
    }
    #[tokio::test]
    async fn it_should_fail_create_one_item() {
        let bodies = vec![
            r#"
        {
            "-1": {
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": 1
            }
        }"#,
            r#"{
            "1": {
                "name": "",
                "notes": "Notes from waiter",
                "quantity": 1
            }
        }"#,
            r#"{
            "1": {
                "name": "Name from menu",
                "notes": "",
                "quantity": -1
            }
        }"#,
        ];
        join_all(bodies.into_iter().map(|b| async move {
            println!("Body {}", b);
            let repo: InMemoryRepository = InMemoryRepository::new();
            let context: Arc<dyn Repository> = Arc::new(repo);
            let mut request = hyper::Request::builder()
                .method(http::Method::POST)
                .uri("/tables/1/items")
                .body(hyper::Body::from(b.to_string()))
                .unwrap();
            request.extensions_mut().insert(context);
            let response = handle(request).await;

            assert_eq!(response.status(), HttpStatus::BAD_REQUEST);
        }))
        .await;
    }

    #[tokio::test]
    async fn it_should_return_ok_create_multi_item() {
        let repo: InMemoryRepository = InMemoryRepository::new();
        let context: Arc<dyn Repository> = Arc::new(repo);
        let body = r#"
        {
            "1": {
                "name": "Name from menu1",
                "notes": "Notes from waiter1",
                "quantity": 1
            },
            "2": {
                "name": "Name from menu2",
                "notes": "Notes from waiter2",
                "quantity": 1
            }
        }"#;
        let mut request = hyper::Request::builder()
            .method(http::Method::POST)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status().clone(), HttpStatus::MULTI_STATUS);
        let body: hyper::body::Bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let binding = String::from_utf8(body.to_vec()).unwrap();
        let result: &str = binding.as_ref();
        //let json: Value = serde_json::from_str(result.as_str()).unwrap();

        join_all(["1", "2"].iter().map(|id| async move {
            let json: Value = serde_json::from_str(result).unwrap();
            assert_eq!(json[id]["status"], HttpStatus::CREATED.as_u16());
            assert_eq!(json[id]["body"]["name"], format!("Name from menu{}", id));
            assert_eq!(
                json[id]["body"]["notes"],
                format!("Notes from waiter{}", id)
            );
            assert_eq!(json[id]["body"]["quantity"], 1);
            assert_eq!(json[id]["body"]["version"], 1);
            assert_eq!(json[id]["body"]["deleted"], false);
            assert!(json[id]["body"]["time_to_prepare"] != "");
        }))
        .await;
    }

    #[tokio::test]
    async fn it_should_return_error_for_second_item_create_multi_item() {
        let repo: InMemoryRepository = InMemoryRepository::new();
        let context: Arc<dyn Repository> = Arc::new(repo);
        let body = r#"
        {
            "1": {
                "name": "Name from menu1",
                "notes": "Notes from waiter1",
                "quantity": 1
            },
            "2": {
                "name": "",
                "notes": "Notes from waiter2",
                "quantity": 2
            }
        }"#;
        let mut request = hyper::Request::builder()
            .method(http::Method::POST)
            .uri("/tables/1/items")
            .body(hyper::Body::from(body))
            .unwrap();
        request.extensions_mut().insert(context);
        let response = handle(request).await;

        assert_eq!(response.status(), HttpStatus::BAD_REQUEST);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(result.as_str()).unwrap();
        assert!(json["msg"]
            .to_string()
            .contains("An error at item with id: 2:"));
    }

    #[tokio::test]
    async fn test_404_url() {
        let request = hyper::Request::builder()
            .method(http::Method::GET)
            .uri("/nonsense")
            .body(hyper::Body::empty())
            .unwrap();
        let response = handle(request).await;

        assert_eq!(response.status(), http::status::StatusCode::NOT_FOUND);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let result = String::from_utf8(body.to_vec()).unwrap();
        let document = Document::from(result.as_str());
        let message = document.find(Name("h1")).next().unwrap().text();
        assert_eq!(message, "NOT FOUND!".to_owned());
    }
}
