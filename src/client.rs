use rand::Rng;
use reqwest;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Client of the restaurant API

pub fn client_main(id: u16, base_url: &str, sleep_max_ms: u64, is_running: Arc<AtomicBool>) {
    tracing::info!("Spawned a client with id {}", id);
    while is_running.load(Ordering::Relaxed) {

        let methods = vec!["GET", "POST", "PUT", "DELETE"];
        let mut rng = rand::thread_rng();

        for _ in 0..4 {
            let method = methods[rng.gen_range(0..methods.len())];
            let endpoint = match method {
                "GET" => {
                    if rng.gen_bool(0.5) {
                        format!(
                            "{}/tables/{}/items",
                            base_url,
                            gen_random_table_id(&mut rng)
                        )
                    } else {
                        let table_id = gen_random_table_id(&mut rng);
                        let item_id = gen_random_item_id(&mut rng);
                        format!("{}/tables/{}/items/{}", base_url, table_id, item_id)
                    }
                }
                "POST" => format!(
                    "{}/tables/{}/items",
                    base_url,
                    gen_random_table_id(&mut rng)
                ),
                "PUT" => format!(
                    "{}/tables/{}/items",
                    base_url,
                    gen_random_table_id(&mut rng)
                ),
                "DELETE" => gen_delete_url(base_url, &mut rng),
                _ => panic!("Invalid method"),
            };

            let method_clone = method.to_string();
            let body = match method_clone.as_str() {
                "POST" => Some(gen_post_body(&mut rng)),
                "PUT" => Some(gen_put_body(&mut rng)),
                "DELETE" => Some(gen_delete_body(&endpoint, &mut rng)),
                _ => None,
            };

            make_request_sync(id, method, &endpoint, body);

            // Introduce some delay between requests
            thread::sleep(Duration::from_millis(rng.gen_range(0..sleep_max_ms)));
        }
    }
    tracing::info!("Exited client {}", id);
}

fn gen_random_number(rng: &mut rand::rngs::ThreadRng) -> usize {
    rng.gen_range(1..=100)
}

fn gen_random_table_id(rng: &mut rand::rngs::ThreadRng) -> usize {
    gen_random_number(rng)
}

fn gen_random_item_id(rng: &mut rand::rngs::ThreadRng) -> usize {
    gen_random_number(rng)
}

fn gen_post_body(rng: &mut rand::rngs::ThreadRng) -> String {
    let mut entries = HashMap::new();
    let num_entries = rng.gen_range(1..=10);

    for _ in 1..num_entries {
        let item_id = gen_random_item_id(rng);
        entries.insert(
            item_id.to_string(),
            json!({
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": rng.gen_range(1..=10)
            }),
        );
    }

    json!(entries).to_string()
}

fn gen_put_body(rng: &mut rand::rngs::ThreadRng) -> String {
    let mut entries = HashMap::new();
    let num_entries = rng.gen_range(1..=10);

    for _ in 1..num_entries {
        let item_id = gen_random_item_id(rng);
        entries.insert(
            item_id.to_string(),
            json!({
                "name": "Name from menu",
                "notes": "Notes from waiter",
                "quantity": rng.gen_range(1..=10),
                "version": rng.gen_range(1..=1000)
            }),
        );
    }

    json!(entries).to_string()
}

fn gen_delete_body(url: &str, rng: &mut rand::rngs::ThreadRng) -> String {
    if url.ends_with("items") {
        let num_ids = rng.gen_range(1..=10);
        let ids: Vec<usize> = (0..num_ids).map(|_| gen_random_item_id(rng)).collect();

        return json!({
            "ids": ids
        })
        .to_string();
    }
    return String::from("");
}

fn gen_delete_url(base_url: &str, rng: &mut rand::rngs::ThreadRng) -> String {
    let table_id = gen_random_table_id(rng);
    if rng.gen_bool(0.5) {
        format!("{}/tables/{}/items", base_url, table_id)
    } else {
        let item_id = gen_random_item_id(rng);
        format!("{}/tables/{}/items/{}", base_url, table_id, item_id)
    }
}

fn make_request_sync(client_id: u16, method: &str, url: &str, body: Option<String>) {
    let client = reqwest::blocking::Client::new();
    let mut request_builder = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => panic!("Invalid method"),
    };

    if let Some(body_content) = body {
        request_builder = request_builder.body(body_content);
    }

    let response = request_builder.timeout(Duration::from_secs(5)).send();

    match response {
        Ok(res) => {
            tracing::info!("C[{}] {} {}: {:?}", client_id, method, url, res.status());
        }
        Err(err) => {
            tracing::info!("C[{}] {} {} failed: {:?}", client_id, method, url, err);
        }
    }
}
