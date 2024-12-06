use std::net::{Ipv4Addr, Ipv6Addr};

use axum::{extract::Query, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct DestParameters {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

pub async fn dest(params: Query<DestParameters>) -> impl IntoResponse {
    let params = params.0;
    let from = params.from.octets();
    let key = params.key.octets();

    let mut res: [u8; 4] = [0; 4];
    for i in 0..4 {
        res[i] = from[i].overflowing_add(key[i]).0;
    }

    Ipv4Addr::from(res).to_string()
}

#[derive(serde::Deserialize)]
pub struct KeyParameters {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

pub async fn key(params: Query<KeyParameters>) -> impl IntoResponse {
    let params = params.0;
    let from = params.from.octets();
    let to = params.to.octets();
    let mut res: [u8; 4] = [0; 4];
    for i in 0..res.len() {
        res[i] = to[i].overflowing_sub(from[i]).0;
    }

    Ipv4Addr::from(res).to_string()
}

#[derive(serde::Deserialize)]
pub struct V6DestParameters {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

pub async fn v6_dest(params: Query<V6DestParameters>) -> impl IntoResponse {
    let params = params.0;
    let from = params.from.octets();
    let key = params.key.octets();
    let mut res: [u8; 16] = [0; 16];
    for i in 0..res.len() {
        res[i] = from[i] ^ key[i];
    }

    Ipv6Addr::from(res).to_string()
}

#[derive(serde::Deserialize)]
pub struct V6KeyParameters {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

pub async fn v6_key(params: Query<V6KeyParameters>) -> impl IntoResponse {
    let params = params.0;
    let from = params.from.octets();
    let to = params.to.octets();
    let mut res: [u8; 16] = [0; 16];
    for i in 0..res.len() {
        res[i] = to[i] ^ from[i];
    }

    Ipv6Addr::from(res).to_string()
}
