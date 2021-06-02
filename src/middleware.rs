use std::task::{Context, Poll};

use crate::cache::RssCache;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, HttpResponse};
use futures::future::{ok, Ready, Either};
use actix_web::web::Data;
use std::sync::Mutex;

pub struct Cache;

impl<S, B> Transform<S> for Cache
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CacheMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CacheMiddleware { service })
    }
}

pub struct CacheMiddleware<S> {
    service: S,
}

impl<S, B> Service for CacheMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future =  Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let key = req.path().to_string();
        eprintln!("key = {:#?}", key);
        let cache = req.app_data::<Data<Mutex<RssCache>>>().unwrap().to_owned();
        let mut cache = cache.lock().unwrap();
        println!("cache len: {}", cache.channel.len());
        if let Some(channel) = cache.get_channel(&key) {
            eprintln!("get cache");
            Either::Right(ok(req.into_response(
                HttpResponse::Ok()
                    .header(http::header::CONTENT_TYPE, "application/xml")
                    .body(channel.to_string())
                    .into_body(),
            )))
        } else {
            eprintln!("no cache");
            Either::Left(self.service.call(req))
        }
    }
}
