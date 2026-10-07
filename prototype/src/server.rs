use std::net::SocketAddr;
use std::sync::Arc;

use app_core::api::VacationAppApi;
use app_core::app_ext::{
    vacation_summary, AdminEmployeeDetail, EmployeePermissions, EmployeeProfile, GroupDetail,
    GroupMemberPermissions, MockPaymentDetails, VacationAppExt, VacationGroupApproval,
};
use app_core::auth::Session;
use app_core::employees::{EmployeeID, VacationID};
use app_core::organizations::OrganizationID;
use app_core::session_cookie::{
    build_clear_cookie, build_set_cookie, read_optional_sessions, read_session_cookie,
    validate_cookie_session, DualSessions, SessionInfo, SessionKind,
};
use app_core::users::UserID;
use app_core::Date;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::PrototypeVacationApp;

#[derive(Clone)]
struct AppState {
    app: Arc<PrototypeVacationApp>,
}

#[derive(Serialize)]
struct HealthResponse {
    backend: String,
    database: String,
    status: &'static str,
}

#[derive(Serialize)]
struct EmployeeResponse {
    id: u32,
    username: String,
    email: String,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
struct UserResponse {
    id: u32,
    email: String,
    organization_id: u32,
}

#[derive(Serialize)]
struct OrganizationResponse {
    id: u32,
    owner_id: u32,
    name: String,
    employee_ids: Vec<u32>,
}

#[derive(Serialize)]
struct VacationResponse {
    id: u32,
    employee_id: u32,
    start: String,
    end: String,
    status: String,
    details: String,
    denial_reason: String,
    approved_by: Option<u32>,
    denied_by: Option<u32>,
    covered_by: Option<u32>,
    cancellation_requested_by: Option<u32>,
    cancelled_by: Option<u32>,
    group_approvals: Vec<VacationGroupApproval>,
}

#[derive(Serialize)]
struct VacationEventResponse {
    id: u32,
    event_type: String,
    actor_id: Option<u32>,
    employee_id: Option<u32>,
    vacation_id: Option<u32>,
    payload: String,
    created_at: String,
}

#[derive(Deserialize)]
struct RoleQuery {
    role: String,
}

#[derive(Deserialize)]
struct OptionalRoleQuery {
    role: Option<String>,
}

#[derive(Deserialize)]
struct UpdateAvatarRequest {
    /// Data URL (`data:image/...;base64,...`) or null/omit to clear.
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct CreateEmployeeRequest {
    username: String,
    email: String,
    organization_id: Option<u32>,
    password: Option<String>,
}

#[derive(Deserialize)]
struct UpdateEmployeeRequest {
    username: String,
    email: String,
    password: Option<String>,
    password_change_email: Option<String>,
}

#[derive(Deserialize)]
struct SetEmployeeGroupsRequest {
    group_ids: Vec<u32>,
}

#[derive(Deserialize)]
struct CreateOrganizationRequest {
    owner_id: u32,
    name: String,
}

#[derive(Deserialize)]
struct UpdateOrganizationRequest {
    name: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    role: String,
}

#[derive(Deserialize)]
struct RequestVacationBody {
    employee_id: Option<u32>,
    start_year: u8,
    start_month: u8,
    start_day: u8,
    final_year: u8,
    final_month: u8,
    final_day: u8,
    details: Option<String>,
}

#[derive(Deserialize)]
struct ApproveVacationRequest {
    covered_by: Option<u32>,
    group_id: Option<u32>,
}

#[derive(Deserialize)]
struct DenyVacationRequest {
    group_id: Option<u32>,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct UpdateVacationBody {
    start_year: u8,
    start_month: u8,
    start_day: u8,
    final_year: u8,
    final_month: u8,
    final_day: u8,
    details: Option<String>,
}

#[derive(Deserialize)]
struct CreateGroupRequest {
    name: String,
}

#[derive(Deserialize)]
struct UpdateGroupRequest {
    name: String,
}

#[derive(Deserialize)]
struct GroupMemberRequest {
    employee_id: u32,
}

#[derive(Deserialize)]
struct AddOrgEmployeeRequest {
    employee_id: u32,
}

struct ApiError(app_core::api::AppError);

impl From<app_core::api::AppError> for ApiError {
    fn from(error: app_core::api::AppError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            app_core::api::AppError::NotFound => StatusCode::NOT_FOUND,
            app_core::api::AppError::InvalidData => StatusCode::BAD_REQUEST,
            app_core::api::AppError::Forbidden => StatusCode::FORBIDDEN,
            app_core::api::AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message = match &self.0 {
            app_core::api::AppError::InvalidData => {
                "End date must be after the start date.".to_string()
            }
            other => other.to_string(),
        };
        (status, message).into_response()
    }
}

pub async fn run_localhost() -> Result<(), app_core::api::AppError> {
    let app = PrototypeVacationApp::load()?;
    let addr = app.config().server.socket_addr();
    run_with_app(addr, Arc::new(app)).await
}

pub async fn run(addr: SocketAddr) -> Result<(), app_core::api::AppError> {
    let app = Arc::new(PrototypeVacationApp::load()?);
    run_with_app(addr, app).await
}

async fn run_with_app(
    addr: SocketAddr,
    app: Arc<PrototypeVacationApp>,
) -> Result<(), app_core::api::AppError> {
    app.setup().await?;

    let state = AppState { app };
    let backend = state.app.config().prototype.backend_name.clone();
    let db_name = state.app.config().db_name().to_string();
    let email_host = state.app.config().email.smtp_host.clone();
    let email_port = state.app.config().email.smtp_port;
    let email_from = state.app.config().email.from_address.clone();
    let email_tls = state.app.config().email.tls;

    let router = Router::new()
        .route("/", get(home))
        .route("/admin", get(admin_page))
        .route("/employee", get(employee_page))
        .route("/static/app.css", get(static_css))
        .route("/static/common.js", get(static_common_js))
        .route("/static/admin.js", get(static_admin_js))
        .route("/static/employee.js", get(static_employee_js))
        .route("/health", get(health))
        .route("/setup", post(setup))
        .route("/employees", get(list_employees).post(create_employee))
        .route("/employees/me", get(my_employee_profile).put(update_my_account))
        .route("/employees/me/avatar", put(update_my_avatar))
        .route("/employees/{employee_id}", put(update_employee).delete(delete_employee))
        .route("/employees/{employee_id}/detail", get(admin_employee_detail))
        .route("/employees/{employee_id}/permissions", get(get_employee_permissions).put(set_employee_permissions))
        .route("/employees/{employee_id}/groups", put(set_employee_groups))
        .route("/users/me", get(my_user_profile))
        .route("/users/me/payment", get(get_my_payment).put(set_my_payment))
        .route("/organizations", get(list_organizations).post(create_organization))
        .route("/organizations/{org_id}", put(update_organization))
        .route(
            "/organizations/{org_id}/employees",
            post(add_org_employee),
        )
        .route("/organizations/{org_id}/groups", get(list_groups).post(create_group))
        .route("/groups/{group_id}", put(update_group).delete(delete_group_route))
        .route("/groups/{group_id}/members", post(add_group_member))
        .route("/groups/{group_id}/members/{employee_id}", axum::routing::delete(remove_group_member))
        .route(
            "/groups/{group_id}/members/{employee_id}/permissions",
            get(get_group_member_permissions).put(set_group_member_permissions),
        )
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(current_session))
        .route("/auth/sessions", get(list_sessions))
        .route("/vacations", get(list_vacations).post(request_vacation))
        .route("/vacations/mine", get(my_vacations))
        .route("/vacations/{vacation_id}", put(update_vacation).delete(delete_vacation_route))
        .route("/vacations/{vacation_id}/events", get(list_vacation_events))
        .route("/vacations/{vacation_id}/approve", post(approve_vacation))
        .route("/vacations/{vacation_id}/deny", post(deny_vacation))
        .route(
            "/vacations/{vacation_id}/request-cancellation",
            post(request_vacation_cancellation),
        )
        .route(
            "/vacations/{vacation_id}/approve-cancellation",
            post(approve_vacation_cancellation),
        )
        .route(
            "/vacations/{vacation_id}/deny-cancellation",
            post(deny_vacation_cancellation),
        )
        .route("/events", get(list_all_events))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        let reason = if err.kind() == std::io::ErrorKind::AddrInUse {
            "address already in use".to_string()
        } else {
            err.to_string()
        };
        app_core::api::AppError::BindFailed {
            addr: addr.to_string(),
            reason,
        }
    })?;

    println!("prototype demo listening on http://{addr}");
    println!("  backend:  {backend}");
    println!("  database: {db_name} (via PostgresConnector)");
    println!(
        "  email:    {}:{} (from {}) tls={:?}",
        email_host, email_port, email_from, email_tls
    );
    println!("  config:   prototype_configs.toml");
    println!("  home:     http://{addr}/");
    println!("  admin:    http://{addr}/admin  (demo@test.com / demo)");
    println!("  employee: http://{addr}/employee  (bob@test.com / demo)");

    axum::serve(listener, router)
        .await
        .map_err(|_| app_core::api::AppError::QueryFailed)
}

fn cookie_header(headers: &HeaderMap) -> &str {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
}

fn json_with_cookie<T: Serialize>(body: T, cookie: String) -> Response {
    let mut response = Json(body).into_response();
    response
        .headers_mut()
        .append(header::SET_COOKIE, cookie.parse().expect("cookie header"));
    response
}

fn status_with_cookie(status: StatusCode, cookie: String) -> Response {
    let mut response = status.into_response();
    response
        .headers_mut()
        .append(header::SET_COOKIE, cookie.parse().expect("cookie header"));
    response
}

async fn require_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(Session, UserID), ApiError> {
    let info = read_session_cookie(cookie_header(headers), SessionKind::User)?;
    let session = validate_cookie_session(state.app.as_ref(), &info).await?;
    Ok((session, info.user_id()?))
}

async fn require_employee(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(Session, EmployeeID), ApiError> {
    let info = read_session_cookie(cookie_header(headers), SessionKind::Employee)?;
    let session = validate_cookie_session(state.app.as_ref(), &info).await?;
    Ok((session, info.employee_id()?))
}

async fn home() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn admin_page() -> Html<&'static str> {
    Html(include_str!("../static/admin.html"))
}

async fn employee_page() -> Html<&'static str> {
    Html(include_str!("../static/employee.html"))
}

async fn static_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], include_str!("../static/app.css"))
}

async fn static_common_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/common.js"),
    )
}

async fn static_admin_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/admin.js"),
    )
}

async fn static_employee_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/employee.js"),
    )
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        backend: state.app.config().prototype.backend_name.clone(),
        database: state.app.config().db_name().to_string(),
        status: "ok",
    })
}

async fn setup(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    state.app.setup().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_employees(
    State(state): State<AppState>,
    Query(query): Query<OptionalRoleQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<EmployeeResponse>>, ApiError> {
    let employees = match query.role.as_deref().map(SessionKind::from_role) {
        Some(SessionKind::Employee) => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            state.app.list_employees_visible_to(employee_id).await?
        }
        Some(SessionKind::User) => {
            let _ = require_user(&state, &headers).await?;
            state.app.list_employees().await?
        }
        None => state.app.list_employees().await?,
    };
    Ok(Json(
        employees
            .into_iter()
            .map(|employee| EmployeeResponse {
                id: employee.employee_id().id(),
                username: employee.username().to_string(),
                email: employee.email().to_string(),
                avatar_url: employee.avatar_url().map(str::to_string),
            })
            .collect(),
    ))
}

async fn create_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateEmployeeRequest>,
) -> Result<Json<EmployeeProfile>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    let org_id = body
        .organization_id
        .ok_or(app_core::api::AppError::InvalidData)?;
    Ok(Json(
        state
            .app
            .create_employee_for_org(
                user_id,
                org_id,
                &body.username,
                &body.email,
                body.password.as_deref().unwrap_or("demo"),
            )
            .await?,
    ))
}

async fn admin_employee_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(employee_id): Path<u32>,
) -> Result<Json<AdminEmployeeDetail>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    Ok(Json(
        state
            .app
            .admin_get_employee_detail(user_id, EmployeeID::new(employee_id))
            .await?,
    ))
}

async fn update_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(employee_id): Path<u32>,
    Json(body): Json<UpdateEmployeeRequest>,
) -> Result<Json<EmployeeProfile>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    Ok(Json(
        state
            .app
            .admin_update_employee(
                user_id,
                EmployeeID::new(employee_id),
                &body.username,
                &body.email,
                body.password.as_deref(),
            )
            .await?,
    ))
}

async fn delete_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(employee_id): Path<u32>,
) -> Result<StatusCode, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    state
        .app
        .admin_delete_employee(user_id, EmployeeID::new(employee_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_employee_groups(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(employee_id): Path<u32>,
    Json(body): Json<SetEmployeeGroupsRequest>,
) -> Result<Json<EmployeeProfile>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    Ok(Json(
        state
            .app
            .admin_set_employee_groups(
                user_id,
                EmployeeID::new(employee_id),
                body.group_ids,
            )
            .await?,
    ))
}

async fn my_user_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    let user = state.app.get_user(user_id).await?;
    Ok(Json(UserResponse {
        id: user.user_id().id(),
        email: user.email().to_string(),
        organization_id: user.organization_id(),
    }))
}

async fn my_employee_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<EmployeeProfile>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    Ok(Json(state.app.get_employee_profile(employee_id).await?))
}

async fn update_my_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateEmployeeRequest>,
) -> Result<Json<EmployeeProfile>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    Ok(Json(
        state
            .app
            .update_own_account(
                employee_id,
                &body.username,
                &body.email,
                body.password.as_deref(),
                body.password_change_email.as_deref(),
            )
            .await?,
    ))
}

async fn update_my_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateAvatarRequest>,
) -> Result<Json<EmployeeProfile>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    Ok(Json(
        state
            .app
            .update_own_avatar(employee_id, body.avatar_url.as_deref())
            .await?,
    ))
}

async fn get_my_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MockPaymentDetails>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    Ok(Json(state.app.get_user_payment(user_id).await?))
}

async fn set_my_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MockPaymentDetails>,
) -> Result<Json<MockPaymentDetails>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    Ok(Json(
        state.app.set_user_payment(user_id, body).await?,
    ))
}

async fn list_organizations(
    State(state): State<AppState>,
) -> Result<Json<Vec<OrganizationResponse>>, ApiError> {
    let organizations = state.app.list_organizations().await?;
    Ok(Json(
        organizations
            .into_iter()
            .map(|org| OrganizationResponse {
                id: org.organization_id().id(),
                owner_id: org.owner_id(),
                name: org.name().to_string(),
                employee_ids: org.employee_ids().to_vec(),
            })
            .collect(),
    ))
}

async fn create_organization(
    State(state): State<AppState>,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<Json<OrganizationResponse>, ApiError> {
    let org = state
        .app
        .create_organization(body.owner_id, &body.name)
        .await?;
    Ok(Json(OrganizationResponse {
        id: org.organization_id().id(),
        owner_id: org.owner_id(),
        name: org.name().to_string(),
        employee_ids: org.employee_ids().to_vec(),
    }))
}

async fn update_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<u32>,
    Json(body): Json<UpdateOrganizationRequest>,
) -> Result<Json<OrganizationResponse>, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    let org = state
        .app
        .admin_update_organization(user_id, OrganizationID::new(org_id), &body.name)
        .await?;
    Ok(Json(OrganizationResponse {
        id: org.organization_id().id(),
        owner_id: org.owner_id(),
        name: org.name().to_string(),
        employee_ids: org.employee_ids().to_vec(),
    }))
}

async fn add_org_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<u32>,
    Json(body): Json<AddOrgEmployeeRequest>,
) -> Result<StatusCode, ApiError> {
    let (_, user_id) = require_user(&state, &headers).await?;
    let user = state.app.get_user(user_id).await?;
    if user.organization_id() != org_id {
        return Err(ApiError(app_core::api::AppError::Forbidden));
    }
    state
        .app
        .add_employee_to_organization(
            OrganizationID::new(org_id),
            EmployeeID::new(body.employee_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_groups(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(org_id): Path<u32>,
) -> Result<Json<Vec<GroupDetail>>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            let user = state.app.get_user(user_id).await?;
            if user.organization_id() != org_id {
                return Err(ApiError(app_core::api::AppError::Forbidden));
            }
            Ok(Json(state.app.list_group_details(org_id).await?))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            let profile = state.app.get_employee_profile(employee_id).await?;
            if profile.organization_id != Some(org_id) {
                return Err(ApiError(app_core::api::AppError::Forbidden));
            }
            Ok(Json(state.app.list_group_details(org_id).await?))
        }
    }
}

async fn create_group(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(org_id): Path<u32>,
    Json(body): Json<CreateGroupRequest>,
) -> Result<Json<GroupDetail>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_create_group(user_id, org_id, &body.name)
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .create_group_as_employee(employee_id, org_id, &body.name)
                    .await?,
            ))
        }
    }
}

async fn update_group(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(group_id): Path<u32>,
    Json(body): Json<UpdateGroupRequest>,
) -> Result<Json<GroupDetail>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_update_group(user_id, group_id, &body.name)
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .update_group_as_employee(employee_id, group_id, &body.name)
                    .await?,
            ))
        }
    }
}

async fn delete_group_route(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(group_id): Path<u32>,
) -> Result<StatusCode, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            state.app.admin_delete_group(user_id, group_id).await?;
            Ok(StatusCode::NO_CONTENT)
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            state
                .app
                .delete_group_as_employee(employee_id, group_id)
                .await?;
            Ok(StatusCode::NO_CONTENT)
        }
    }
}

async fn add_group_member(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(group_id): Path<u32>,
    Json(body): Json<GroupMemberRequest>,
) -> Result<Json<GroupDetail>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_add_group_member(user_id, group_id, EmployeeID::new(body.employee_id))
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .add_group_member(
                        employee_id,
                        group_id,
                        EmployeeID::new(body.employee_id),
                    )
                    .await?,
            ))
        }
    }
}

async fn remove_group_member(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path((group_id, member_id)): Path<(u32, u32)>,
) -> Result<Json<GroupDetail>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_remove_group_member(user_id, group_id, EmployeeID::new(member_id))
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .remove_group_member(employee_id, group_id, EmployeeID::new(member_id))
                    .await?,
            ))
        }
    }
}

async fn get_group_member_permissions(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path((group_id, member_id)): Path<(u32, u32)>,
) -> Result<Json<GroupMemberPermissions>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_get_group_member_permissions(
                        user_id,
                        group_id,
                        EmployeeID::new(member_id),
                    )
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .get_group_member_permissions(
                        employee_id,
                        group_id,
                        EmployeeID::new(member_id),
                    )
                    .await?,
            ))
        }
    }
}

async fn set_group_member_permissions(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path((group_id, member_id)): Path<(u32, u32)>,
    Json(body): Json<GroupMemberPermissions>,
) -> Result<Json<GroupMemberPermissions>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_set_group_member_permissions(
                        user_id,
                        group_id,
                        EmployeeID::new(member_id),
                        body,
                    )
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .set_group_member_permissions(
                        employee_id,
                        group_id,
                        EmployeeID::new(member_id),
                        body,
                    )
                    .await?,
            ))
        }
    }
}

async fn get_employee_permissions(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(employee_id): Path<u32>,
) -> Result<Json<EmployeePermissions>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            let detail = state
                .app
                .admin_get_employee_detail(user_id, EmployeeID::new(employee_id))
                .await?;
            Ok(Json(detail.profile.permissions))
        }
        SessionKind::Employee => {
            let (_, actor_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .get_employee_permissions(actor_id, EmployeeID::new(employee_id))
                    .await?,
            ))
        }
    }
}

async fn set_employee_permissions(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(employee_id): Path<u32>,
    Json(body): Json<EmployeePermissions>,
) -> Result<Json<EmployeePermissions>, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .admin_set_employee_permissions(
                        user_id,
                        EmployeeID::new(employee_id),
                        body,
                    )
                    .await?,
            ))
        }
        SessionKind::Employee => {
            let (_, actor_id) = require_employee(&state, &headers).await?;
            Ok(Json(
                state
                    .app
                    .set_employee_permissions(actor_id, EmployeeID::new(employee_id), body)
                    .await?,
            ))
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    let session = match body.role.as_str() {
        "employee" => {
            state
                .app
                .authenticate_employee(&body.email, &body.password)
                .await?
        }
        _ => state
            .app
            .authenticate_user(&body.email, &body.password)
            .await?,
    };

    let info = SessionInfo::from_session(&session);
    let cookie = build_set_cookie(&info);
    Ok(json_with_cookie(info, cookie))
}

async fn logout(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let kind = SessionKind::from_role(&query.role);
    if let Ok(info) = read_session_cookie(cookie_header(&headers), kind) {
        state.app.logout(info.session_id).await?;
    }
    Ok(status_with_cookie(
        StatusCode::NO_CONTENT,
        build_clear_cookie(kind),
    ))
}

async fn current_session(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
) -> Result<Json<SessionInfo>, ApiError> {
    let kind = SessionKind::from_role(&query.role);
    let info = read_session_cookie(cookie_header(&headers), kind)?;
    validate_cookie_session(state.app.as_ref(), &info).await?;
    Ok(Json(info))
}

/// Returns both portal sessions the browser currently holds (admin and/or employee).
async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DualSessions>, ApiError> {
    let raw = read_optional_sessions(cookie_header(&headers));
    let mut dual = DualSessions::default();

    if let Some(info) = raw.admin {
        if validate_cookie_session(state.app.as_ref(), &info).await.is_ok() {
            dual.admin = Some(info);
        }
    }
    if let Some(info) = raw.employee {
        if validate_cookie_session(state.app.as_ref(), &info).await.is_ok() {
            dual.employee = Some(info);
        }
    }

    Ok(Json(dual))
}

async fn list_vacations(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<VacationResponse>>, ApiError> {
    let vacations = match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            let vacations = state.app.admin_list_org_vacations(user_id).await?;
            return Ok(Json(enrich_vacations(&state, vacations).await?));
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            let vacations = state.app.list_vacations_visible_to(employee_id).await?;
            let mut responses = enrich_vacations(&state, vacations).await?;
            redact_vacation_details_for_employee(&state, employee_id, &mut responses).await?;
            Ok(Json(responses))
        }
    };
    vacations
}

async fn my_vacations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<VacationResponse>>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    let vacations = state.app.get_vacations_for_employee(employee_id).await?;
    Ok(Json(enrich_vacations(&state, vacations).await?))
}

async fn request_vacation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RequestVacationBody>,
) -> Result<Json<VacationResponse>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    let target_id = body
        .employee_id
        .map(EmployeeID::new)
        .unwrap_or(employee_id);

    if target_id != employee_id {
        return Err(ApiError(app_core::api::AppError::Forbidden));
    }

    let start = Date::new(body.start_year, body.start_month, body.start_day);
    let end = Date::new(body.final_year, body.final_month, body.final_day);
    if !vacation_end_after_start(start, end) {
        return Err(ApiError(app_core::api::AppError::InvalidData));
    }

    let vacation = state
        .app
        .request_vacation_with_permissions(
            target_id,
            start,
            end,
            body.details.as_deref().unwrap_or(""),
        )
        .await?;
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn approve_vacation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
    Query(query): Query<OptionalRoleQuery>,
    body: Result<Json<ApproveVacationRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<VacationResponse>, ApiError> {
    let vacation = match query.role.as_deref().map(SessionKind::from_role) {
        Some(SessionKind::User) => {
            let (_, user_id) = require_user(&state, &headers).await?;
            state
                .app
                .admin_approve_vacation(user_id, VacationID::new(vacation_id))
                .await?
        }
        _ => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            let Json(req) = body.unwrap_or(Json(ApproveVacationRequest {
                covered_by: None,
                group_id: None,
            }));
            let covered_by = req.covered_by.map(EmployeeID::new).unwrap_or(employee_id);
            state
                .app
                .approve_vacation_with_permissions(
                    employee_id,
                    VacationID::new(vacation_id),
                    covered_by,
                    req.group_id,
                )
                .await?
        }
    };
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn deny_vacation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
    Query(query): Query<OptionalRoleQuery>,
    body: Result<Json<DenyVacationRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<VacationResponse>, ApiError> {
    let body = body.ok().map(|Json(b)| b);
    let reason = body.as_ref().and_then(|b| b.reason.as_deref());
    let group_id = body.as_ref().and_then(|b| b.group_id);
    let vacation = match query.role.as_deref().map(SessionKind::from_role) {
        Some(SessionKind::User) => {
            let (_, user_id) = require_user(&state, &headers).await?;
            state
                .app
                .admin_deny_vacation(user_id, VacationID::new(vacation_id), reason)
                .await?
        }
        _ => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            state
                .app
                .deny_vacation_with_permissions(
                    employee_id,
                    VacationID::new(vacation_id),
                    group_id,
                    reason,
                )
                .await?
        }
    };
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn request_vacation_cancellation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
) -> Result<Json<VacationResponse>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    let vacation = state
        .app
        .request_vacation_cancellation_with_permissions(employee_id, VacationID::new(vacation_id))
        .await?;
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn approve_vacation_cancellation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
    body: Result<Json<DenyVacationRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<VacationResponse>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    let group_id = body.ok().and_then(|Json(b)| b.group_id);
    let vacation = state
        .app
        .approve_vacation_cancellation_with_permissions(
            employee_id,
            VacationID::new(vacation_id),
            group_id,
        )
        .await?;
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn deny_vacation_cancellation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
    body: Result<Json<DenyVacationRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<VacationResponse>, ApiError> {
    let (_, employee_id) = require_employee(&state, &headers).await?;
    let group_id = body.ok().and_then(|Json(b)| b.group_id);
    let vacation = state
        .app
        .deny_vacation_cancellation_with_permissions(
            employee_id,
            VacationID::new(vacation_id),
            group_id,
        )
        .await?;
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn update_vacation(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
    Json(body): Json<UpdateVacationBody>,
) -> Result<Json<VacationResponse>, ApiError> {
    let start = Date::new(body.start_year, body.start_month, body.start_day);
    let end = Date::new(body.final_year, body.final_month, body.final_day);
    if !vacation_end_after_start(start, end) {
        return Err(ApiError(app_core::api::AppError::InvalidData));
    }
    let vacation = match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            state
                .app
                .admin_update_vacation(
                    user_id,
                    VacationID::new(vacation_id),
                    start,
                    end,
                    body.details.as_deref().unwrap_or(""),
                )
                .await?
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            state
                .app
                .update_vacation_with_permissions(
                    employee_id,
                    VacationID::new(vacation_id),
                    start,
                    end,
                    body.details.as_deref().unwrap_or(""),
                )
                .await?
        }
    };
    Ok(Json(to_vacation_response_async(&state, vacation).await?))
}

async fn delete_vacation_route(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
) -> Result<StatusCode, ApiError> {
    match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            state
                .app
                .admin_delete_vacation(user_id, VacationID::new(vacation_id))
                .await?;
        }
        SessionKind::Employee => {
            let (_, employee_id) = require_employee(&state, &headers).await?;
            state
                .app
                .delete_vacation_with_permissions(employee_id, VacationID::new(vacation_id))
                .await?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn list_vacation_events(
    State(state): State<AppState>,
    Query(query): Query<RoleQuery>,
    headers: HeaderMap,
    Path(vacation_id): Path<u32>,
) -> Result<Json<Vec<VacationEventResponse>>, ApiError> {
    let events = match SessionKind::from_role(&query.role) {
        SessionKind::User => {
            let (_, user_id) = require_user(&state, &headers).await?;
            state
                .app
                .admin_list_vacation_events(user_id, VacationID::new(vacation_id))
                .await?
        }
        SessionKind::Employee => {
            let _ = require_employee(&state, &headers).await?;
            state
                .app
                .list_vacation_events(VacationID::new(vacation_id))
                .await?
        }
    };
    Ok(Json(
        events
            .into_iter()
            .map(|event| VacationEventResponse {
                id: event.id,
                event_type: event.event_type,
                actor_id: event.actor_id,
                employee_id: event.employee_id,
                vacation_id: event.vacation_id,
                payload: event.payload,
                created_at: event.created_at,
            })
            .collect(),
    ))
}

async fn list_all_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<VacationEventResponse>>, ApiError> {
    let _ = require_user(&state, &headers).await?;
    let events = state.app.list_all_events().await?;
    Ok(Json(
        events
            .into_iter()
            .map(|event| VacationEventResponse {
                id: event.id,
                event_type: event.event_type,
                actor_id: event.actor_id,
                employee_id: event.employee_id,
                vacation_id: event.vacation_id,
                payload: event.payload,
                created_at: event.created_at,
            })
            .collect(),
    ))
}

fn to_vacation_response(
    vacation: app_core::employees::Vacation,
    group_approvals: Vec<VacationGroupApproval>,
) -> VacationResponse {
    let summary = vacation_summary(&vacation);
    VacationResponse {
        id: summary.id,
        employee_id: vacation.employee_id().id(),
        start: summary.start,
        end: summary.end,
        status: summary.status,
        details: vacation.details().to_string(),
        denial_reason: vacation.denial_reason().to_string(),
        approved_by: vacation.approved_by().map(|id| id.id()),
        denied_by: vacation.denied_by().map(|id| id.id()),
        covered_by: vacation.covered_by().map(|id| id.id()),
        cancellation_requested_by: vacation.cancellation_requested_by().map(|id| id.id()),
        cancelled_by: vacation.cancelled_by().map(|id| id.id()),
        group_approvals,
    }
}

async fn to_vacation_response_async(
    state: &AppState,
    vacation: app_core::employees::Vacation,
) -> Result<VacationResponse, ApiError> {
    let approvals = state
        .app
        .vacation_group_approvals(vacation.vacation_id())
        .await?;
    Ok(to_vacation_response(vacation, approvals))
}

async fn enrich_vacations(
    state: &AppState,
    vacations: Vec<app_core::employees::Vacation>,
) -> Result<Vec<VacationResponse>, ApiError> {
    let ids: Vec<u32> = vacations.iter().map(|v| v.vacation_id().id()).collect();
    let grouped = state.app.vacation_group_approvals_for_many(&ids).await?;
    let mut by_id: std::collections::HashMap<u32, Vec<VacationGroupApproval>> =
        grouped.into_iter().collect();
    Ok(vacations
        .into_iter()
        .map(|vacation| {
            let id = vacation.vacation_id().id();
            let approvals = by_id.remove(&id).unwrap_or_default();
            to_vacation_response(vacation, approvals)
        })
        .collect())
}

async fn redact_vacation_details_for_employee(
    state: &AppState,
    viewer_id: app_core::employees::EmployeeID,
    responses: &mut [VacationResponse],
) -> Result<(), ApiError> {
    for response in responses.iter_mut() {
        let group_ids: Vec<u32> = response
            .group_approvals
            .iter()
            .map(|vote| vote.group_id)
            .collect();
        let can_see = state
            .app
            .can_see_vacation_details(viewer_id, response.employee_id, &group_ids)
            .await?;
        if !can_see {
            response.details.clear();
        }
    }
    Ok(())
}

fn vacation_end_after_start(start: Date, end: Date) -> bool {
    (end.year(), end.month(), end.day()) > (start.year(), start.month(), start.day())
}
