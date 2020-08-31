#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_files as fs;
use actix_session::{CookieSession, Session};
use actix_utils::mpsc;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Result,
};
use bytes::Bytes;
use async_trait::async_trait;


struct Data {
    name: String
}

#[async_trait]
trait Service {
    async fn get(&self, data: &Data) -> Vec<u8>;
}

struct ServiceImpl {
    nc: nats::asynk::Connection,
}

#[async_trait]
impl Service for ServiceImpl {
    async fn get(&self, data: &Data) -> Vec<u8> {
        let result = self.nc.request("demo", data.name.as_bytes()).await.unwrap();
        result.data
    }
}

async fn use_nats(service: web::Data<Box<dyn Service>>) -> HttpResponse {
    let data = Data {
        name: "demo".to_owned(),
    };

    let result = service.get(&data).await;

    HttpResponse::Ok().body(result)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let nc = nats::asynk::connect("0.0.0.0:4222").await.unwrap();

    HttpServer::new(move || {
        App::new()
            // async response body
            .app_data(Box::new(ServiceImpl {
                nc: nc.clone()
            }))
            .service(
                web::resource("/nats").route(web::get().to(use_nats)),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}