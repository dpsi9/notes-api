use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{HeaderName, HeaderValue},
};

use futures_util::future::{LocalBoxFuture, Ready, ready};
use uuid::Uuid;

pub struct RequestId;

impl<S, B> Transform<S, ServiceRequest> for RequestId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddleware { service }))
    }
}

pub struct RequestIdMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let rid = Uuid::new_v4().to_string();
        req.extensions_mut().insert(rid.clone());

        let method = req.method().clone();
        let path = req.path().to_string();
        tracing::info!(request_id = %rid, %method, %path, "incoming request");

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            if let Ok(val) = HeaderValue::from_str(&rid) {
                res.headers_mut()
                    .insert(HeaderName::from_static("x-request-id"), val);
            }
            Ok(res)
        })
    }
}
