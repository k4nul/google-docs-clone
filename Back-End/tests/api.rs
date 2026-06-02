//! Integration tests against a running server process.
//!
//! Start the server first:
//!   cargo run
//!
//! Then run these tests:
//!   cargo test --test api -- --ignored
//!
//! Override defaults with env vars:
//!   TEST_BASE_URL   (default: http://localhost:4000)
//!   TEST_API_TOKEN  (default: dev-admin-token)

use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

fn base_url() -> String {
    std::env::var("TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:4000".to_owned())
}

fn admin_auth() -> String {
    let token = std::env::var("TEST_API_TOKEN").unwrap_or_else(|_| "dev-admin-token".to_owned());
    format!("Bearer {token}")
}

fn doc_auth(token: &str) -> String {
    format!("Bearer {token}")
}

// ── Health ────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_health_endpoint_returns_ok() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/health", base_url()))
        .send()
        .await
        .expect("health request should reach the server");

    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.expect("response should be JSON");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "backend");
    assert!(body["timestamp"].as_str().is_some());
}

// ── Document CRUD ─────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_create_list_get_delete_document() {
    let client = Client::new();
    let base = base_url();
    let auth = admin_auth();

    let create_resp = client
        .post(format!("{base}/api/documents"))
        .header("Authorization", &auth)
        .json(&serde_json::json!({ "title": "Localhost CRUD test" }))
        .send()
        .await
        .expect("create request should succeed");
    assert_eq!(create_resp.status().as_u16(), 201);
    let create_body: Value = create_resp.json().await.unwrap();
    let doc_id = create_body["document"]["id"]
        .as_str()
        .expect("document id should be present")
        .to_owned();
    let access_token = create_body["credentials"]["access_token"]
        .as_str()
        .expect("access token should be present")
        .to_owned();
    assert_eq!(
        create_body["document"]["title"].as_str(),
        Some("Localhost CRUD test")
    );
    assert!(create_body["document"]["access_token"].is_null());

    let list_resp = client
        .get(format!("{base}/api/documents"))
        .header("Authorization", &auth)
        .send()
        .await
        .expect("list request should succeed");
    assert_eq!(list_resp.status().as_u16(), 200);
    let list_body: Value = list_resp.json().await.unwrap();
    let ids: Vec<&str> = list_body["documents"]
        .as_array()
        .expect("documents should be an array")
        .iter()
        .filter_map(|d| d["id"].as_str())
        .collect();
    assert!(
        ids.contains(&doc_id.as_str()),
        "created document should appear in list"
    );

    let get_resp = client
        .get(format!("{base}/api/documents/{doc_id}"))
        .header("Authorization", doc_auth(&access_token))
        .send()
        .await
        .expect("get request should succeed");
    assert_eq!(get_resp.status().as_u16(), 200);
    let get_body: Value = get_resp.json().await.unwrap();
    assert_eq!(get_body["document"]["id"].as_str(), Some(doc_id.as_str()));
    assert_eq!(
        get_body["document"]["title"].as_str(),
        Some("Localhost CRUD test")
    );
    assert!(get_body["document"]["access_token"].is_null());

    let update_resp = client
        .patch(format!("{base}/api/documents/{doc_id}"))
        .header("Authorization", doc_auth(&access_token))
        .json(&serde_json::json!({ "title": "Localhost renamed test" }))
        .send()
        .await
        .expect("update request should succeed");
    assert_eq!(update_resp.status().as_u16(), 200);
    let update_body: Value = update_resp.json().await.unwrap();
    assert_eq!(
        update_body["document"]["title"].as_str(),
        Some("Localhost renamed test")
    );

    let delete_resp = client
        .delete(format!("{base}/api/documents/{doc_id}"))
        .header("Authorization", doc_auth(&access_token))
        .send()
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_resp.status().as_u16(), 204);

    let get_after_delete_resp = client
        .get(format!("{base}/api/documents/{doc_id}"))
        .header("Authorization", doc_auth(&access_token))
        .send()
        .await
        .expect("get-after-delete request should succeed");
    assert_eq!(get_after_delete_resp.status().as_u16(), 404);
    let not_found_body: Value = get_after_delete_resp.json().await.unwrap();
    assert_eq!(not_found_body["error"], "not_found");
}

// ── Auth error cases ──────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_documents_list_rejects_missing_authorization() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/documents", base_url()))
        .send()
        .await
        .expect("request should reach the server");

    assert_eq!(resp.status().as_u16(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "unauthorized");
    assert_eq!(body["message"], "Authorization header is required");
}

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_documents_list_rejects_wrong_admin_token() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/documents", base_url()))
        .header("Authorization", "Bearer wrong-admin-token")
        .send()
        .await
        .expect("request should reach the server");

    assert_eq!(resp.status().as_u16(), 403);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "forbidden");
    assert_eq!(
        body["message"],
        "provided API token does not grant this operation"
    );
}

// ── Validation error cases ────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_document_detail_rejects_invalid_uuid() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/documents/not-a-uuid", base_url()))
        .header("Authorization", "Bearer some-token")
        .send()
        .await
        .expect("request should reach the server");

    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert_eq!(
        body["message"],
        "id must be a valid UUID, received `not-a-uuid`"
    );
}

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_document_detail_returns_not_found_for_missing_document() {
    let client = Client::new();
    let doc_id = Uuid::nil();
    let resp = client
        .get(format!("{}/api/documents/{doc_id}", base_url()))
        .header("Authorization", "Bearer some-token")
        .send()
        .await
        .expect("request should reach the server");

    assert_eq!(resp.status().as_u16(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "not_found");
    assert_eq!(
        body["message"],
        format!("document `{doc_id}` was not found")
    );
}

#[tokio::test]
#[ignore = "requires a running server — cargo run, then: cargo test --test api -- --ignored"]
async fn localhost_create_document_assigns_default_title_when_title_is_absent() {
    let client = Client::new();
    let resp = client
        .post(format!("{}/api/documents", base_url()))
        .header("Authorization", admin_auth())
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("request should reach the server");

    assert_eq!(resp.status().as_u16(), 201);
    let body: Value = resp.json().await.unwrap();
    let id = body["document"]["id"]
        .as_str()
        .expect("document id should be returned");
    let title = body["document"]["title"]
        .as_str()
        .expect("document title should be returned");
    assert_eq!(title, format!("Document {id}"));

    // clean up
    let access_token = body["credentials"]["access_token"]
        .as_str()
        .expect("access token should be returned")
        .to_owned();
    Client::new()
        .delete(format!("{}/api/documents/{id}", base_url()))
        .header("Authorization", doc_auth(&access_token))
        .send()
        .await
        .ok();
}
