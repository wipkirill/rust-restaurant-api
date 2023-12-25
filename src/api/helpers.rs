use crate::domain::types::{
    IdType, Item, ItemId, ItemName, ItemNotes, ItemQuantity, ItemVersion, VersionType,
};
use hyper::http;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urlpattern::Error;
use urlpattern::UrlPattern;
use urlpattern::UrlPatternInit;
use urlpattern::UrlPatternMatchInput;
use urlpattern::UrlPatternResult;

// This file contains helper structs and functions to read 
// data from requests and output to responses

#[derive(Debug, Serialize, Deserialize)]
pub struct NewItem {
    pub name: String,
    pub notes: String,
    pub quantity: i32,
    pub version: Option<i32>,
}
#[derive(Serialize)]
pub struct FailMsg {
    pub msg: String,
}

#[derive(Serialize)]
pub struct MultiOperationResponse<T> {
    pub status: String,
    pub body: T,
}

#[derive(Debug, Deserialize)]
pub struct ItemIdsList {
    pub ids: Vec<ItemId<IdType>>,
}

#[derive(Serialize)]
pub struct StatusWithBody {
    pub status: u16,
    pub body: serde_json::value::Value,
}

pub type CreateOrUpdateItemRequest = HashMap<ItemId<IdType>, NewItem>;
pub type OpStatusResponse = HashMap<ItemId<IdType>, StatusWithBody>;
pub type OpItemsResponse = HashMap<ItemId<IdType>, serde_json::value::Value>;

pub type Request = http::Request<hyper::Body>;
pub type Response = http::Response<hyper::Body>;

pub fn match_url(url: &str, pattern: &str) -> bool {
    match_url_result(url, pattern).unwrap().is_some()
}

pub fn match_url_result(url: &str, pattern: &str) -> Result<Option<UrlPatternResult>, Error> {
    // Create the UrlPattern to match against.
    let init = UrlPatternInit {
        pathname: Some(pattern.to_owned()),
        ..Default::default()
    };

    let pattern = <UrlPattern>::parse(init).unwrap();

    // Match the pattern against a URL.
    let url_init: UrlPatternInit = UrlPatternInit {
        pathname: Some(url.to_string()),
        ..Default::default()
    };
    pattern.exec(UrlPatternMatchInput::Init(url_init))
}

pub async fn four_oh_four() -> Response {
    html_str_handler("<h1>NOT FOUND!</h1>", http::StatusCode::NOT_FOUND).await
}

async fn html_str_handler(html: &str, status_code: http::StatusCode) -> Response {
    string_handler(html, "text/html", status_code).await
}

pub async fn to_json<T>(obj: T, status_code: http::StatusCode) -> Response
where
    T: Serialize,
{
    match serde_json::to_string(&obj) {
        Ok(json_string) => string_handler(&json_string, "application/json", status_code).await,
        Err(err) => {
            // Handle the serialization error, return a 500 Internal Server Error for simplicity
            string_handler(
                &format!("Internal Server Error: {}", err),
                "text/plain",
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .await
        }
    }
}

async fn string_handler(body: &str, content_type: &str, status_code: http::StatusCode) -> Response {
    hyper::Response::builder()
        .status(status_code)
        .header(hyper::header::CONTENT_TYPE, content_type)
        .body(hyper::Body::from(body.to_string()))
        .unwrap_or_else(|_| {
            hyper::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(hyper::Body::empty())
                .unwrap()
        })
}

pub async fn extract_string_payload(request: Request) -> Result<String, String> {
    let body = request.into_body();
    let bytes_buf = hyper::body::to_bytes(body)
        .await
        .map_err(|err| err.to_string())?;
    String::from_utf8(bytes_buf.to_vec()).map_err(|err| err.to_string())
}

pub fn parse_delete_items_request(data: &str) -> Result<Vec<ItemId<IdType>>, String> {
    let req_items: ItemIdsList = serde_json::from_str(data).map_err(|err| err.to_string())?;

    for id in &req_items.ids {
        let _ = ItemId::try_from(id.to_string())
            .map_err(|err| format!("Cannot parse item with id: {}: {}", id, err))?;
    }

    match req_items.ids.len() {
        0 => Err("An empty list for deletion provided".to_string()),
        _ => Ok(req_items.ids),
    }
}
// Used to parse items from POST and PUT json data with validation
pub fn parse_create_or_update_items(data: &str) -> Result<Vec<Item>, String> {
    let items_map: CreateOrUpdateItemRequest =
        serde_json::from_str(data).map_err(|err| err.to_string())?;

    let mut items: Vec<Item> = Vec::new();
    let err_tpl = |idx, err| format!("An error at item with id: {}: {}", idx, err);

    for (idx, new_item) in items_map {
        let item_id: ItemId<IdType> =
            ItemId::try_from(idx.to_string()).map_err(|err| err_tpl(idx, err))?;
        let name: ItemName = ItemName::try_from(new_item.name).map_err(|err| err_tpl(idx, err))?;
        let notes: ItemNotes =
            ItemNotes::try_from(new_item.notes).map_err(|err| err_tpl(idx, err))?;
        let quantity: ItemQuantity<u32> = ItemQuantity::try_from(new_item.quantity.to_string())
            .map_err(|err| err_tpl(idx, err))?;
        let item_version_str: String = new_item.version.unwrap_or_else(|| 1).to_string();
        let item_version: ItemVersion<VersionType> =
            ItemVersion::try_from(item_version_str).map_err(|err| err_tpl(idx, err))?;

        let it: Item = Item {
            id: item_id,
            name,
            notes,
            quantity,
            deleted: false,
            version: item_version,
            time_to_prepare: "".to_string(),
        };

        items.push(it);
    }

    match items.len() {
        0 => Err("An empty body provided".to_string()),
        _ => Ok(items),
    }
}

// Read table and item id from string url
pub fn parse_numeric_id<T>(url: &str, url_pattern: &str, numeric_group: &str) -> Result<T, String>
where
    T: TryFrom<String, Error = String>,
{
    let match_url_result = match_url_result(&url, url_pattern).map_err(|err| err.to_string())?;
    let numeric_id_str = match_url_result
        .unwrap()
        .pathname
        .groups
        .get(numeric_group)
        .ok_or_else(|| format!("Numeric group '{}' not found", numeric_group))?
        .clone();
    Ok(T::try_from(numeric_id_str).map_err(|err| format!("{}", err))?)
}

pub fn json_body<T: Serialize>(body: T) -> serde_json::Value {
    serde_json::to_value(body).expect("Failed to convert type to JSON value")
}

#[cfg(test)]
mod tests {
    use crate::domain::types::{IdType, TableId};

    use super::{parse_create_or_update_items, parse_delete_items_request, parse_numeric_id};

    #[test]
    fn a_parse_numeric_ids_from_url() {
        let mut url = "/tables/1/items/1";
        let pattern = "/tables/:tid/items/:id";

        match parse_numeric_id::<TableId<IdType>>(url, pattern, "tid") {
            Ok(_) => {}
            _ => unreachable!(),
        };

        url = "/tables/abc/items/1";
        match parse_numeric_id::<TableId<IdType>>(url, pattern, "tid") {
            Ok(_) => unreachable!(),
            _ => {}
        };

        url = "/tables/0/items/1";
        match parse_numeric_id::<TableId<IdType>>(url, pattern, "tid") {
            Ok(_) => unreachable!(),
            _ => {}
        };
    }

    #[test]
    fn a_parse_array_item_ids() {
        let mut data = r#"
        {
            "ids": [0, 2, 3]
        }"#;
        match parse_delete_items_request(data) {
            Err(_) => {}
            _ => unreachable!(),
        };

        data = r#"
        {
            "ids": [1, 2, 3]
        }"#;
        match parse_delete_items_request(data) {
            Err(_) => unreachable!(),
            _ => {}
        };
    }

    #[test]
    fn a_parse_create_update_items() {
        let mut data = r#"
        {
            "0": {
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": 100
            }
        }"#;
        match parse_create_or_update_items(data) {
            Err(_) => {}
            _ => unreachable!(),
        };

        data = r#"
        {
            "1": {
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": 100
            },
            "2": {
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": 100
            }
        }"#;
        match parse_create_or_update_items(data) {
            Err(_) => unreachable!(),
            _ => {}
        };
    }
}
