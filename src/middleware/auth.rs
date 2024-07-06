use axum::{
    extract::{Request, State},
    http::{self},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::headers::authorization::{Bearer, Credentials};

use crate::{
    service::auth_service::AuthService,
    util::api_response::{ApiError, ServiceError},
};

pub async fn authorize(
    State(service): State<AuthService>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let auth_header_value = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(ServiceError::MissingAuthToken)?;

    let auth_token = Bearer::decode(auth_header_value).ok_or(ServiceError::InvalidAuthToken)?;

    let auth_info = service.extract_auth_info(auth_token.token()).await?;
    // Insert auth info for further use
    req.extensions_mut().insert(auth_info);
    Ok(next.run(req).await)
}

#[macro_export]
macro_rules! permission_required {
    ($($perm:expr),+ $(,)?) => {{
        use $crate::util::api_response::ServiceError;
        use axum::{
            extract::Request,
            middleware::{from_fn, Next},
            response::IntoResponse,
            Extension,
        };
        async fn check(Extension(auth_info): Extension<AuthInfo>, req: Request, next: Next) -> Result<impl IntoResponse, ApiError> {
            $(
                if (!auth_info.permissions.contains(&$perm.into())) {
                    return Err(ServiceError::MissingRequiredPermission(String::from($perm.as_str())).into());
                }
            )+
            Ok(next.run(req).await)
        }

        from_fn(check)
    }};
}
