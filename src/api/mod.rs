use hyper::http;

pub mod server;
pub mod create_items;
pub mod delete_items;
pub mod helpers;
pub mod read_item;
pub mod update_items;

use crate::api::helpers::{four_oh_four, match_url};
use create_items::create_items_handler;
use delete_items::{delete_item_handler, delete_items_handler};
use read_item::{read_item_handler, read_items_handler};
use update_items::update_items_handler;
pub type Request = http::Request<hyper::Body>;
pub type Response = http::Response<hyper::Body>;

use hyper::StatusCode as HttpStatus;
/// Handles incoming HTTP requests and dispatches them to specific handlers based on the HTTP method and URI path.
///
/// # Arguments
///
/// * `request` - An HTTP request object containing information about the incoming request.
///
/// # Returns
///
/// An asynchronous `Response` object representing the HTTP response to be sent back to the client.
pub async fn handle(request: Request) -> Response {
    // pattern match for both the method and the path of the request
    match (request.method(), request.uri().path()) {
        // Delete
        (m, s) if m.eq(&hyper::Method::DELETE) && match_url(&s, "/tables/:tid/items/:id") => {
            delete_item_handler(request).await
        }
        (m, s) if m.eq(&hyper::Method::DELETE) && match_url(&s, "/tables/:tid/items") => {
            delete_items_handler(request).await
        }
        // Get
        (m, s) if m.eq(&hyper::Method::GET) && match_url(&s, "/tables/:tid/items/:id") => {
            read_item_handler(request).await
        }
        (m, s) if m.eq(&hyper::Method::GET) && match_url(&s, "/tables/:tid/items") => {
            read_items_handler(request).await
        }

        //Update
        (m, s) if m.eq(&hyper::Method::PUT) && match_url(&s, "/tables/:tid/items") => {
            update_items_handler(request).await
        }
        //Create
        (m, s) if m.eq(&hyper::Method::POST) && match_url(&s, "/tables/:tid/items") => {
            create_items_handler(request).await
        }

        // Anything else
        _ => four_oh_four().await,
    }
}
