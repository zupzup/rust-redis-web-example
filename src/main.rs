use mobc_pool::MobcPool;
use r2d2_pool::R2D2Pool;
use std::convert::Infallible;
use thiserror::Error;
use warp::{Filter, Rejection, Reply};

mod direct;
mod mobc_pool;
mod r2d2_pool;

type WebResult<T> = std::result::Result<T, Rejection>;
type Result<T> = std::result::Result<T, Error>;

const REDIS_CON_STRING: &str = "redis://127.0.0.1/";

#[tokio::main]
async fn main() {
    let redis_client = redis::Client::open(REDIS_CON_STRING).expect("can create redis client");
    let mobc_pool = mobc_pool::connect().await.expect("can create mobc pool");
    let r2d2_pool = r2d2_pool::connect().expect("can create r2d2 pool");

    let direct_route = warp::path!("direct")
        .and(with_redis_client(redis_client.clone()))
        .and_then(direct_handler);

    let mobc_route = warp::path!("mobc")
        .and(with_mobc_pool(mobc_pool.clone()))
        .and_then(mobc_handler);

    let r2d2_route = warp::path!("r2d2")
        .and(with_r2d2_pool(r2d2_pool.clone()))
        .and_then(r2d2_handler);

    let routes = mobc_route.or(direct_route).or(r2d2_route);

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

async fn mobc_handler(pool: MobcPool) -> WebResult<impl Reply> {
    mobc_pool::set_str(&pool, "mobc_hello", "mobc_world", 60)
        .await
        .map_err(|e| warp::reject::custom(e))?;
    let value = mobc_pool::get_str(&pool, "mobc_hello")
        .await
        .map_err(|e| warp::reject::custom(e))?;
    Ok(value)
}

async fn r2d2_handler(pool: R2D2Pool) -> WebResult<impl Reply> {
    r2d2_pool::set_str(&pool, "r2d2_hello", "r2d2_world", 60)
        .map_err(|e| warp::reject::custom(e))?;
    let value = r2d2_pool::get_str(&pool, "r2d2_hello").map_err(|e| warp::reject::custom(e))?;
    Ok(value)
}

async fn direct_handler(client: redis::Client) -> WebResult<impl Reply> {
    let mut con = direct::get_con(client)
        .await
        .map_err(|e| warp::reject::custom(e))?;
    direct::set_str(&mut con, "hello", "direct_world", 60)
        .await
        .map_err(|e| warp::reject::custom(e))?;
    let value = direct::get_str(&mut con, "hello")
        .await
        .map_err(|e| warp::reject::custom(e))?;
    Ok(value)
}

fn with_redis_client(
    client: redis::Client,
) -> impl Filter<Extract = (redis::Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_mobc_pool(
    pool: MobcPool,
) -> impl Filter<Extract = (MobcPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn with_r2d2_pool(
    pool: R2D2Pool,
) -> impl Filter<Extract = (R2D2Pool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("mobc error: {0}")]
    MobcError(#[from] MobcError),
    #[error("direct redis error: {0}")]
    DirectError(#[from] DirectError),
    #[error("r2d2 error: {0}")]
    R2D2Error(#[from] R2D2Error),
}

#[derive(Error, Debug)]
pub enum MobcError {
    #[error("could not get redis connection from pool : {0}")]
    RedisPoolError(mobc::Error<mobc_redis::redis::RedisError>),
    #[error("error parsing string from redis result: {0}")]
    RedisTypeError(mobc_redis::redis::RedisError),
    #[error("error executing redis command: {0}")]
    RedisCMDError(mobc_redis::redis::RedisError),
    #[error("error creating Redis client: {0}")]
    RedisClientError(mobc_redis::redis::RedisError),
}

#[derive(Error, Debug)]
pub enum R2D2Error {
    #[error("could not get redis connection from pool : {0}")]
    RedisPoolError(r2d2_redis::r2d2::Error),
    #[error("error parsing string from redis result: {0}")]
    RedisTypeError(r2d2_redis::redis::RedisError),
    #[error("error executing redis command: {0}")]
    RedisCMDError(r2d2_redis::redis::RedisError),
    #[error("error creating Redis client: {0}")]
    RedisClientError(r2d2_redis::redis::RedisError),
}

#[derive(Error, Debug)]
pub enum DirectError {
    #[error("error parsing string from redis result: {0}")]
    RedisTypeError(redis::RedisError),
    #[error("error executing redis command: {0}")]
    RedisCMDError(redis::RedisError),
    #[error("error creating Redis client: {0}")]
    RedisClientError(redis::RedisError),
}

impl warp::reject::Reject for Error {}
