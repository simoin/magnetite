use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http,
    web::Data,
    Error, HttpResponse,
};
use futures::future::{ok, Ready};
use rss::Channel;

use magnetite_cache::Storage;

pub struct Cache;

impl<S, B> Transform<S, ServiceRequest> for Cache
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CacheMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CacheMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct CacheMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for CacheMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        let key = req.path().to_string();
        let cache = req.app_data::<Data<Storage>>().unwrap().clone();

        Box::pin(async move {
            if let Ok(Some(channel)) = cache.get::<_, Channel>(&key).await {
                Ok(req.into_response(
                    HttpResponse::Ok()
                        .append_header((http::header::CONTENT_TYPE, "application/xml"))
                        .body(channel.to_string())
                        .into_body(),
                ))
            } else {
                Ok(svc.call(req).await?)
            }
        })
    }
}
