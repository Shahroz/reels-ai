//! Minimal Rust client for the Imageboard API (single file).
//!
//! Usage:
//! ```ignore
//! use imageboard_backend::imageboard_client::{ImageboardClient, CreateBoardRequest};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ImageboardClient::new("http://localhost:3000").with_api_key("secret-key");
//! // Create with random id
//! let created = client.create_board_admin(CreateBoardRequest::default()).await?;
//! // Or create with predefined UUID
//! let created2 = client.create_board_admin(CreateBoardRequest{ board_id: Some("11111111-1111-4111-8111-111111111111".into()), ..Default::default() }).await?;
//! println!("board id = {}", created.board_id);
//! # Ok(())
//! # }
//! ```
//!
//! Notes:
//! - This module relies on `reqwest` and `serde` crates. Add dependencies:
//!   reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
//!   serde = { version = "1", features = ["derive"] }
//!   serde_json = "1"
//!
//! - Base URL defaults to http://localhost:3000 if not provided. Override as needed.

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

#[derive(Debug, Clone)]
pub struct ImageboardClient {
    base_url: String,
    api_key: Option<String>,
    impersonate_user_id: Option<String>,
    http: reqwest::Client,
}

impl ImageboardClient {
    pub fn new<S: Into<String>>(base_url: S) -> Self {
        Self { base_url: base_url.into().trim_end_matches('/').to_string(), api_key: None, impersonate_user_id: None, http: reqwest::Client::new() }
    }
    pub fn with_api_key<S: Into<String>>(mut self, key: S) -> Self { self.api_key = Some(key.into()); self }
    /// Impersonate a user via `X-API-Key` + `X-User-Id` headers (server middleware issues a dashboard token).
    pub fn with_impersonated_user_id<S: Into<String>>(mut self, user_id: S) -> Self { self.impersonate_user_id = Some(user_id.into()); self }

    fn url(&self, path: &str) -> String { format!("{}{}{}", self.base_url, if path.starts_with('/') { "" } else { "/" }, path) }

    // Internal: add API key header if configured; add impersonation headers if both api_key and user_id are set
    fn maybe_impersonate(&self, mut rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api) = &self.api_key {
            rb = rb.header("X-API-Key", api.clone());
        }
        if let (Some(api), Some(uid)) = (&self.api_key, &self.impersonate_user_id) {
            rb = rb.header("X-User-Id", uid.clone());
        }
        rb
    }

    // Public create
    pub async fn create_board(&self, body: CreateBoardRequest) -> Result<CreateBoardResponse, ClientError> {
        let url = self.url("/api/boards");
        tracing::info!("POST {} with owner_user_id={}, has_api_key={}", url, body.owner_user_id, self.api_key.is_some());
        let req = self.http.post(&url).json(&body);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    // Admin create (supports predefined UUID via body.board_id)
    pub async fn create_board_admin(&self, body: CreateBoardRequest) -> Result<CreateBoardResponse, ClientError> {
        let mut req = self.http.post(self.url("/api/admin/boards"));
        if let Some(k) = &self.api_key { req = req.header("X-API-Key", k); }
        let res = req.json(&body).send().await?;
        handle_json(res).await
    }

    pub async fn get_board(&self, board_id: &str) -> Result<GetBoardResponse, ClientError> {
        let url = self.url(&format!("/api/boards/{}", board_id));
        let req = self.http.get(url);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    pub async fn get_access_tokens(&self, board_id: &str, editor_access_token: &str) -> Result<AccessTokensResponse, ClientError> {
        let enc = utf8_percent_encode(editor_access_token, NON_ALPHANUMERIC).to_string();
        let url = self.url(&format!("/api/boards/{}/access?accessToken={}", board_id, enc));
        let req = self.http.get(url);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    pub async fn get_access_tokens_admin(&self, board_id: &str) -> Result<AccessTokensResponse, ClientError> {
        let mut req = self.http.get(self.url(&format!("/api/admin/boards/{}/access", board_id)));
        if let Some(k) = &self.api_key { req = req.header("X-API-Key", k); }
        let res = req.send().await?;
        handle_json(res).await
    }

    /// Construct a shareable URL to open the board in the web app for a given token.
    pub fn build_board_url(&self, board_id: &str, access_token: &str) -> String {
        let mut base = self.base_url.clone();
        if base.ends_with('/') { base.pop(); }
        let tok = utf8_percent_encode(access_token, NON_ALPHANUMERIC).to_string();
        format!("{}/?boardId={}&accessToken={}", base, board_id, tok)
    }

    /// Construct a shareable URL including a login token for auto-login.
    pub fn build_board_url_with_login(&self, board_id: &str, access_token: &str, login_token: &str) -> String {
        let mut base = self.base_url.clone();
        if base.ends_with('/') { base.pop(); }
        let tok = utf8_percent_encode(access_token, NON_ALPHANUMERIC).to_string();
        let ltok = utf8_percent_encode(login_token, NON_ALPHANUMERIC).to_string();
        format!("{}/?boardId={}&accessToken={}&loginToken={}", base, board_id, tok, ltok)
    }

    /// Convenience wrapper: create (public) and return tokens plus computed URLs.
    pub async fn create_board_with_urls(&self, body: CreateBoardRequest) -> Result<CreateBoardResult, ClientError> {
        let r = self.create_board(body).await?;
        let edit_url = if let Some(ref l) = r.login_token {
            self.build_board_url_with_login(&r.board_id, &r.edit_access_token, l)
        } else {
            self.build_board_url(&r.board_id, &r.edit_access_token)
        };
        let view_url = self.build_board_url(&r.board_id, &r.view_access_token);
        Ok(CreateBoardResult { board_id: r.board_id, edit_access_token: r.edit_access_token, view_access_token: r.view_access_token, edit_url, view_url, login_token: r.login_token, amount_cents: r.amount_cents })
    }

    /// Convenience wrapper: admin create (supports predefined UUID) and return tokens + URLs.
    pub async fn create_board_admin_with_urls(&self, body: CreateBoardRequest) -> Result<CreateBoardResult, ClientError> {
        let r = self.create_board_admin(body).await?;
        let edit_url = if let Some(ref l) = r.login_token {
            self.build_board_url_with_login(&r.board_id, &r.edit_access_token, l)
        } else {
            self.build_board_url(&r.board_id, &r.edit_access_token)
        };
        let view_url = self.build_board_url(&r.board_id, &r.view_access_token);
        Ok(CreateBoardResult { board_id: r.board_id, edit_access_token: r.edit_access_token, view_access_token: r.view_access_token, edit_url, view_url, login_token: r.login_token, amount_cents: r.amount_cents })
    }

    // =============== Authentication (user) ===============
    pub async fn auth_signup(&self, req: SignupRequest) -> Result<AuthResponse, ClientError> {
        let res = self.http.post(self.url("/api/auth/signup")).json(&req).send().await?;
        handle_json(res).await
    }
    pub async fn auth_login(&self, req: LoginRequest) -> Result<AuthResponse, ClientError> {
        let res = self.http.post(self.url("/api/auth/login")).json(&req).send().await?;
        handle_json(res).await
    }
    pub async fn auth_refresh(&self, req: RefreshRequest) -> Result<AuthResponse, ClientError> {
        let res = self.http.post(self.url("/api/auth/refresh")).json(&req).send().await?;
        handle_json(res).await
    }
    pub async fn auth_logout(&self, req: LogoutRequest) -> Result<LogoutResponse, ClientError> {
        let res = self.http.post(self.url("/api/auth/logout")).json(&req).send().await?;
        handle_json(res).await
    }
    pub async fn auth_me(&self, bearer: &str) -> Result<MeResponse, ClientError> {
        let res = self.http.get(self.url("/api/auth/me")).bearer_auth(bearer).send().await?;
        handle_json(res).await
    }

    // =============== Admin token ===============
    pub async fn admin_generate_token(&self, req: GenerateAdminTokenRequest) -> Result<GenerateAdminTokenResponse, ClientError> {
        let res = self.http.post(self.url("/api/admin/auth/token")).json(&req).send().await?;
        handle_json(res).await
    }

    // =============== Admin users ===============
    pub async fn admin_list_users(&self, admin_bearer: &str) -> Result<ListUsersResponse, ClientError> {
        let res = self.http.get(self.url("/api/admin/users")).bearer_auth(admin_bearer).send().await?;
        handle_json(res).await
    }
    pub async fn admin_get_user_details(&self, admin_bearer: &str, user_id: &str) -> Result<UserDetailsResponse, ClientError> {
        let res = self.http.get(self.url(&format!("/api/admin/users/{}", user_id))).bearer_auth(admin_bearer).send().await?;
        handle_json(res).await
    }
    pub async fn admin_allocate_credits(&self, admin_bearer: &str, user_id: &str, req: AllocateCreditsRequest) -> Result<AllocateCreditsResponse, ClientError> {
        let res = self.http.post(self.url(&format!("/api/admin/users/{}/credits", user_id))).bearer_auth(admin_bearer).json(&req).send().await?;
        handle_json(res).await
    }
    /// Allocate credits using the master API key (no admin bearer required).
    pub async fn admin_allocate_credits_with_key(&self, user_id: &str, req: AllocateCreditsRequest) -> Result<AllocateCreditsResponse, ClientError> {
        let mut http = self.http.post(self.url(&format!("/api/admin/users/{}/credits", user_id))).json(&req);
        if let Some(k) = &self.api_key { http = http.header("X-API-Key", k); }
        let res = http.send().await?;
        handle_json(res).await
    }
    pub async fn admin_get_user_balance(&self, admin_bearer: &str, user_id: &str) -> Result<UserBalanceResponse, ClientError> {
        let res = self.http.get(self.url(&format!("/api/admin/users/{}/balance", user_id))).bearer_auth(admin_bearer).send().await?;
        handle_json(res).await
    }
    // User's own balance (dashboard token)
    pub async fn get_my_balance(&self, bearer: &str) -> Result<MyBalanceResponse, ClientError> {
        let res = self.http.get(self.url("/api/dashboard/balance")).bearer_auth(bearer).send().await?;
        handle_json(res).await
    }

    // =============== Admin: dashboard access links for users ===============
    /// Generate a dashboard access token and URL for a user (admin bearer auth).
    pub async fn admin_generate_access_link(&self, admin_bearer: &str, req: GenerateAccessLinkRequest) -> Result<GenerateAccessLinkResponse, ClientError> {
        let res = self.http.post(self.url("/api/admin/access-links")).bearer_auth(admin_bearer).json(&req).send().await?;
        handle_json(res).await
    }
    /// Generate a dashboard access token and URL using the master API key (shared secret).
    pub async fn admin_generate_access_link_with_key(&self, req: GenerateAccessLinkRequest) -> Result<GenerateAccessLinkResponse, ClientError> {
        let mut http = self.http.post(self.url("/api/admin/access-links")).json(&req);
        if let Some(k) = &self.api_key { http = http.header("X-API-Key", k); }
        let res = http.send().await?;
        handle_json(res).await
    }
}

async fn handle_json<T: for<'de> Deserialize<'de>>(resp: reqwest::Response) -> Result<T, ClientError> {
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(ClientError::Http(status.as_u16(), text)); }
    serde_json::from_str(&text).map_err(ClientError::from)
}

#[derive(Debug)]
pub enum ClientError {
    Http(u16, String),
    Network(reqwest::Error),
    Json(serde_json::Error),
}

impl From<reqwest::Error> for ClientError { fn from(e: reqwest::Error) -> Self { ClientError::Network(e) } }
impl From<serde_json::Error> for ClientError { fn from(e: serde_json::Error) -> Self { ClientError::Json(e) } }

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Http(code, body) => write!(f, "http {}: {}", code, body),
            ClientError::Network(e) => write!(f, "network error: {}", e),
            ClientError::Json(e) => write!(f, "json error: {}", e),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClientError::Network(e) => Some(e),
            ClientError::Json(e) => Some(e),
            _ => None,
        }
    }
}

// Models (mirrors backend; simplified where optional)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardRequest {
    pub owner_user_id: String,
    #[serde(default)] pub board_id: Option<String>,
    #[serde(default)] pub name: Option<String>,
    #[serde(default)] pub callback_url: Option<String>,
    #[serde(default)] pub check_balance_url: Option<String>,
    #[serde(default)] pub transactions_callback_url: Option<String>,
    #[serde(default)] pub assets: Option<Vec<AssetInput>>,
    // Additional fields supported by the backend
    #[serde(default)] pub is_forkable: Option<bool>,
    #[serde(default)] pub allowed_referrer_url: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub entity_id: Option<String>,
    #[serde(default)] pub email: Option<String>,
    #[serde(default)] pub email_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInput {
    pub url: String,
    #[serde(default)] pub title: Option<String>,
    #[serde(default)] pub width: Option<u32>,
    #[serde(default)] pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardResponse {
    pub board_id: String,
    pub edit_access_token: String,
    pub view_access_token: String,
    #[serde(default)]
    pub login_token: Option<String>,
    #[serde(default)]
    pub amount_cents: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardResult {
    pub board_id: String,
    pub edit_access_token: String,
    pub view_access_token: String,
    pub edit_url: String,
    pub view_url: String,
    #[serde(default)]
    pub login_token: Option<String>,
    #[serde(default)]
    pub amount_cents: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBoardResponse {
    pub server_version: u64,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokensResponse {
    pub board_id: String,
    pub edit_access_token: String,
    pub view_access_token: String,
}

// =============== Dashboard (impersonation-capable) ===============

impl ImageboardClient {
    /// List boards owned by the impersonated user.
    pub async fn dashboard_list_boards(&self, q: Option<&str>, page: Option<u32>, page_size: Option<u32>) -> Result<Vec<BoardListItem>, ClientError> {
        let mut url = self.url("/api/dashboard/boards");
        let mut sep = '?';
        if let Some(s) = q { url.push(sep); url.push_str(&format!("q={}", utf8_percent_encode(s, NON_ALPHANUMERIC))); sep='&'; }
        if let Some(p) = page { url.push(sep); url.push_str(&format!("page={}", p)); sep='&'; }
        if let Some(ps) = page_size { url.push(sep); url.push_str(&format!("page_size={}", ps)); }
        let req = self.http.get(url);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// Create a board as the impersonated user.
    pub async fn dashboard_create_board(&self, req_body: DashboardCreateBoardRequest) -> Result<DashboardCreateBoardResponse, ClientError> {
        let req = self.http.post(self.url("/api/dashboard/boards")).json(&req_body);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// Delete a board as the impersonated user.
    pub async fn dashboard_delete_board(&self, board_id: &str) -> Result<(), ClientError> {
        let req = self.http.delete(self.url(&format!("/api/dashboard/boards/{}", board_id)));
        let res = self.maybe_impersonate(req).send().await?;
        let status = res.status();
        let _ = res.bytes().await; // drain
        if !status.is_success() { return Err(ClientError::Http(status.as_u16(), "".into())); }
        Ok(())
    }

    /// Get owner tokens for a board as the impersonated user.
    pub async fn dashboard_get_board_access(&self, board_id: &str) -> Result<BoardAccessTokensResponse, ClientError> {
        let req = self.http.get(self.url(&format!("/api/dashboard/boards/{}/access", board_id)));
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// Get board analytics for a board as the impersonated user.
    pub async fn dashboard_get_board_stats(&self, board_id: &str, start_date: Option<&str>, end_date: Option<&str>) -> Result<BoardStatsResponse, ClientError> {
        let mut url = self.url(&format!("/api/dashboard/boards/{}/stats", board_id));
        let mut sep = '?';
        if let Some(s) = start_date { url.push(sep); url.push_str(&format!("start_date={}", utf8_percent_encode(s, NON_ALPHANUMERIC))); sep='&'; }
        if let Some(e) = end_date { url.push(sep); url.push_str(&format!("end_date={}", utf8_percent_encode(e, NON_ALPHANUMERIC))); }
        let req = self.http.get(url);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// List embed keys belonging to the impersonated user.
    pub async fn dashboard_list_embed_keys(&self, limit: Option<u32>) -> Result<Vec<EmbedKeyListItem>, ClientError> {
        let mut url = self.url("/api/dashboard/embed-keys");
        if let Some(l) = limit { url.push('?'); url.push_str(&format!("limit={}", l)); }
        let req = self.http.get(url);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// Create an embed key for the impersonated user.
    pub async fn dashboard_create_embed_key(&self, req_body: CreateEmbedKeyRequest) -> Result<CreateEmbedKeyResponse, ClientError> {
        let req = self.http.post(self.url("/api/dashboard/embed-keys")).json(&req_body);
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// Get details for a specific embed key (includes token and example snippet).
    pub async fn dashboard_get_embed_key_details(&self, key_id: &str) -> Result<GetEmbedKeyResponse, ClientError> {
        let req = self.http.get(self.url(&format!("/api/dashboard/embed-keys/{}", key_id)));
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }

    /// Revoke (delete) an embed key by id.
    pub async fn dashboard_revoke_embed_key(&self, key_id: &str) -> Result<(), ClientError> {
        let req = self.http.delete(self.url(&format!("/api/dashboard/embed-keys/{}", key_id)));
        let res = self.maybe_impersonate(req).send().await?;
        let status = res.status();
        let _ = res.bytes().await; // drain
        if !status.is_success() { return Err(ClientError::Http(status.as_u16(), "".into())); }
        Ok(())
    }

    /// Get the impersonated user's balance.
    pub async fn dashboard_get_my_balance(&self) -> Result<MyBalanceResponse, ClientError> {
        let req = self.http.get(self.url("/api/dashboard/balance"));
        let res = self.maybe_impersonate(req).send().await?;
        handle_json(res).await
    }
}

// =============== Models for auth/admin ===============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(default)] pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest { pub email: String, pub password: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest { pub refresh_token: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest { pub refresh_token: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse { pub success: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
    #[serde(default)] pub display_name: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse { pub user_id: String, pub email: Option<String>, pub display_name: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAdminTokenRequest { pub password: String, #[serde(default)] pub duration_days: Option<i64> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAdminTokenResponse { pub token: String, pub expires_at: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersResponse { pub users: Vec<UserListItem> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListItem {
    pub user_id: String,
    pub board_count: i64,
    pub total_sessions: i64,
    pub total_cost_cents: i64,
    pub created_at: String,
    #[serde(default)] pub last_activity_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDetailsResponse { pub user_id: String, pub board_count: usize, pub total_sessions: i64, pub total_cost_cents: i64, pub boards: Vec<BoardSummary> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSummary {
    pub id: String,
    pub name: String,
    pub is_forkable: bool,
    #[serde(default)] pub forked_from_board_id: Option<String>,
    pub created_at: String,
    pub session_count: i64,
    pub unique_visitors: i64,
    pub total_cost_cents: i64,
    #[serde(default)] pub last_activity_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocateCreditsRequest { pub amount_cents: i64, #[serde(default)] pub description: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocateCreditsResponse {
    pub transaction_id: i64, 
    pub new_balance_cents: i64 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalanceResponse {
    pub user_id: String,
    pub balance_cents: i64,
    #[serde(default)] pub credit_count: Option<i64>,
    #[serde(default)] pub debit_count: Option<i64>,
    #[serde(default)] pub total_amount_cents: Option<i64>,
    #[serde(default)] pub total_debits_cents: Option<i64>,
    #[serde(default)] pub last_transaction_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyBalanceResponse { pub user_id: String, pub balance_cents: i64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAccessLinkRequest { pub user_id: String, #[serde(default)] pub duration_days: Option<i64> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAccessLinkResponse { pub user_id: String, pub access_token: String, pub dashboard_url: String, pub expires_at: String }

// Dashboard models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardListItem {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_forkable: bool,
    #[serde(default)] pub allowed_referrer_url: Option<String>,
    #[serde(default)] pub forked_from_board_id: Option<String>,
    #[serde(default)] pub description: Option<String>,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCreateBoardRequest {
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub is_forkable: Option<bool>,
    #[serde(default)] pub allowed_referrer_url: Option<String>,
    #[serde(default)] pub callback_url: Option<String>,
    #[serde(default)] pub check_balance_url: Option<String>,
    #[serde(default)] pub transactions_callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCreateBoardResponse {
    pub board_id: String,
    pub edit_access_token: String,
    pub view_access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardAccessTokensResponse { pub edit_access_token: String, pub view_access_token: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardStatsResponse {
    pub board_id: String,
    pub board_name: String,
    pub daily_stats: Vec<BoardDailyStats>,
    pub totals: StatsTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardDailyStats {
    pub id: i32,
    pub board_id: String,
    pub date: String,
    pub unique_visitor_count: i32,
    pub total_connection_count: i32,
    pub event_count: i32,
    pub enhance_image_count: i32,
    pub enhance_image_cost_cents: i32,
    pub video_generate_count: i32,
    pub video_generate_cost_cents: i32,
    pub total_cost_cents: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsTotals {
    pub total_unique_visitors: i64,
    pub total_connections: i64,
    pub total_events: i64,
    pub total_enhance_image_count: i64,
    pub total_enhance_image_cost_cents: i64,
    pub total_video_generate_count: i64,
    pub total_video_generate_cost_cents: i64,
    pub total_cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedKeyListItem {
    pub id: String,
    pub created_at: String,
    pub expires_at: String,
    #[serde(default)] pub allowed_referrer: Option<String>,
    #[serde(default)] pub allowed_referrer_mode: Option<String>,
    #[serde(default)] pub budget_daily_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmbedKeyRequest {
    #[serde(default)] pub budget_daily_cents: Option<i64>,
    #[serde(default)] pub allowed_referrer: Option<String>,
    #[serde(default)] pub expires_days: Option<u32>,
    #[serde(default)] pub referer_match_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmbedKeyResponse { pub embed_key: String, pub example_snippet: String, pub embed_key_id: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEmbedKeyResponse {
    pub embed_key_id: String,
    pub embed_key: String,
    pub example_snippet: String,
    pub created_at: String,
    pub expires_at: String,
    #[serde(default)] pub allowed_referrer: Option<String>,
    #[serde(default)] pub allowed_referrer_mode: Option<String>,
    #[serde(default)] pub budget_daily_cents: Option<i64>,
}
