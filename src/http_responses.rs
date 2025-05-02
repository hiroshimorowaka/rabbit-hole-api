use actix_web::{HttpResponse, HttpResponseBuilder, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorMessage<'a> {
    status: u16,
    error: &'a str,
}

pub trait HttpResponseExt {
    #[allow(dead_code)]
    fn internal_server_error_default_body(&mut self) -> HttpResponse;
    fn bad_request_default_body(&mut self, message: &str) -> HttpResponse;
    fn not_found_default_body(&mut self, message: &str) -> HttpResponse;
    fn unauthorized_default_body(&mut self) -> HttpResponse;
    fn forbbiden_default_body(&mut self) -> HttpResponse;
    fn conflict_default_body(&mut self, message: &str) -> HttpResponse;
}

impl HttpResponseExt for HttpResponseBuilder {
    fn internal_server_error_default_body(&mut self) -> HttpResponse {
        let msg = ErrorMessage {
            status: 500,
            error: "Internal Server Error",
        };
        self.status(StatusCode::INTERNAL_SERVER_ERROR).json(msg)
    }

    fn bad_request_default_body(&mut self, message: &str) -> HttpResponse {
        let msg = ErrorMessage {
            status: 400,
            error: message,
        };
        self.status(StatusCode::BAD_REQUEST).json(msg)
    }

    fn not_found_default_body(&mut self, message: &str) -> HttpResponse {
        let msg = ErrorMessage {
            status: 404,
            error: message,
        };
        self.status(StatusCode::NOT_FOUND).json(msg)
    }

    fn unauthorized_default_body(&mut self) -> HttpResponse {
        let msg = ErrorMessage {
            status: 401,
            error: "Unauthorized",
        };
        self.status(StatusCode::UNAUTHORIZED).json(msg)
    }

    fn forbbiden_default_body(&mut self) -> HttpResponse {
        let msg = ErrorMessage {
            status: 403,
            error: "Forbidden",
        };
        self.status(StatusCode::FORBIDDEN).json(msg)
    }
    fn conflict_default_body(&mut self, message: &str) -> HttpResponse {
        let msg = ErrorMessage {
            status: 409,
            error: message,
        };
        self.status(StatusCode::CONFLICT).json(msg)
    }
}
