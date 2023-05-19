#![allow(
    clippy::collapsible_else_if,
    clippy::collapsible_match,
    clippy::explicit_into_iter_loop,
    clippy::module_inception,
    clippy::needless_continue,
    clippy::needless_lifetimes,
    clippy::needless_return,
    clippy::new_without_default,
    clippy::redundant_pattern_matching,
    clippy::single_match_else,
    clippy::string_add,
    clippy::too_many_arguments,
    clippy::trait_duplication_in_bounds,
    clippy::unused_unit,
    clippy::empty_enum,
    clippy::let_unit_value
)]
#![deny(
    clippy::unnecessary_cast,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::fallible_impl_from,
    clippy::float_cmp_const,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_for_each,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::rc_mutex,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values
)]

use infrastructure::bb8::Pool;
use infrastructure::bb8_redis::RedisConnectionManager;
use infrastructure::hyper::service::make_service_fn;
use infrastructure::hyper::service::service_fn;
use infrastructure::hyper::Body;
use infrastructure::hyper::Error as HyperError;
use infrastructure::hyper::Method;
use infrastructure::hyper::Request;
use infrastructure::hyper::Response;
use infrastructure::hyper::Server;
use infrastructure::redis::AsyncCommands;
use infrastructure::ConnectionPoolCreator;
use infrastructure::ResponseCreator;
use infrastructure::StorageKeyManager;
use std::convert::From;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use tokio::signal;

const SERVER_SOCKET_ADDRESS: &'static str = "0.0.0.0:80";

fn main() -> () {
    if let Err(error) = run_server() {
        println!("{}", &error);
    }

    return ();
}

#[tokio::main]
async fn run_server() -> Result<(), Box<dyn Error + 'static>> {
    let mut server_socket_address_registry = match SERVER_SOCKET_ADDRESS.to_socket_addrs() {
        Ok(server_socket_address_registry_) => server_socket_address_registry_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let server_socket_address = match server_socket_address_registry.next() {
        Some(server_socket_address_) => server_socket_address_,
        None => {
            return Err(Box::from(LogicError));
        }
    };

    let builder = Server::bind(&server_socket_address);

    let connection_pool = ConnectionPoolCreator::create(5).await?;

    let service = make_service_fn(move |_| {
        let connection_pool_ = connection_pool.clone();

        async move {
            return Ok::<_, HyperError>(service_fn(move |requset| {
                let connection_pool__ = connection_pool_.clone();

                return async move {
                    return Ok::<_, HyperError>(resolve(requset, &connection_pool__).await);
                };
            }));
        }
    });

    if let Err(error) = builder
        .serve(service)
        .with_graceful_shutdown(create_shutdown_signal())
        .await
    {
        return Err(Box::from(error));
    }

    return Ok(());
}

async fn resolve<'a>(
    request: Request<Body>,
    connection_pool: &'a Pool<RedisConnectionManager>,
) -> Response<Body> {
    let route = request.uri().path();

    let method = request.method();

    let response = match (route, method) {
        ("/event/collection_created/quantity", &Method::GET) => {
            get_event_collection_created_quantity(connection_pool).await
        }
        ("/event/collection_created", &Method::GET) => {
            get_event_collection_created_by_index(request, connection_pool).await
        }
        ("/event/token_minted/quantity", &Method::GET) => {
            get_event_token_minted_quantity(connection_pool).await
        }
        ("/event/token_minted", &Method::GET) => {
            get_event_token_minted_by_index(request, connection_pool).await
        }
        _ => {
            return ResponseCreator::create_not_found();
        }
    };

    return response;
}

async fn create_shutdown_signal() -> () {
    signal::ctrl_c()
        .await
        .expect("Failed to install gracefully shutdown signal.");

    return ();
}

fn get_parameter_from_query<'a, T>(query: &'a [u8], key: &'a str) -> Option<T>
where
    T: FromStr,
{
    for (key_, value) in form_urlencoded::parse(query) {
        if key_ == key {
            if let Ok(value_) = value.as_ref().parse() {
                return Some(value_);
            }
        }
    }

    return None;
}

async fn get_event_collection_created_quantity<'a>(
    connection_pool: &'a Pool<RedisConnectionManager>,
) -> Response<Body> {
    let mut pooled_connection = match connection_pool.get().await {
        Ok(pooled_connection_) => pooled_connection_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    let connection = &mut *pooled_connection;

    let quantity = match connection
        .get::<'_, &'_ str, Option<String>>(StorageKeyManager::EVENT_COLLECTION_CREATED_QUANTITY)
        .await
    {
        Ok(quantity_) => quantity_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    match quantity {
        Some(quantity_) => {
            return ResponseCreator::create_ok(Some(quantity_));
        }
        None => {
            return ResponseCreator::create_ok(Some(
                "There are no events of CreateCollection type yet.".to_string(),
            ));
        }
    }
}

async fn get_event_collection_created_by_index<'a>(
    request: Request<Body>,
    connection_pool: &'a Pool<RedisConnectionManager>,
) -> Response<Body> {
    let query = match request.uri().query() {
        Some(query_) => query_,
        None => {
            return ResponseCreator::create_bad_request();
        }
    };

    let index_value = match get_parameter_from_query::<i64>(query.as_bytes(), "index") {
        Some(value_) => value_,
        None => {
            return ResponseCreator::create_bad_request();
        }
    };

    let mut pooled_connection = match connection_pool.get().await {
        Ok(pooled_connection_) => pooled_connection_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    let connection = &mut *pooled_connection;

    let key_event = StorageKeyManager::create_key_event(
        StorageKeyManager::EVENT_COLLECTION_CREATED_PART,
        index_value,
    );

    let data = match connection
        .get::<'_, &'_ str, Option<String>>(&key_event)
        .await
    {
        Ok(data_) => data_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    match data {
        Some(data_) => {
            return ResponseCreator::create_ok(Some(data_));
        }
        None => {
            return ResponseCreator::create_ok(Some(format!(
                "There are no event of CreateCollection type with index {}.",
                index_value
            )))
        }
    }
}

async fn get_event_token_minted_quantity<'a>(
    connection_pool: &'a Pool<RedisConnectionManager>,
) -> Response<Body> {
    let mut pooled_connection = match connection_pool.get().await {
        Ok(pooled_connection_) => pooled_connection_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    let connection = &mut *pooled_connection;

    let quantity = match connection
        .get::<'_, &'_ str, Option<String>>(StorageKeyManager::EVENT_TOKEN_MINTED_QUANTITY)
        .await
    {
        Ok(quantity_) => quantity_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    match quantity {
        Some(quantity_) => {
            return ResponseCreator::create_ok(Some(quantity_));
        }
        None => {
            return ResponseCreator::create_ok(Some(
                "There are no events of TokenMinted type yet.".to_string(),
            ));
        }
    }
}

async fn get_event_token_minted_by_index<'a>(
    request: Request<Body>,
    connection_pool: &'a Pool<RedisConnectionManager>,
) -> Response<Body> {
    let query = match request.uri().query() {
        Some(query_) => query_,
        None => {
            return ResponseCreator::create_bad_request();
        }
    };

    let index_value = match get_parameter_from_query::<i64>(query.as_bytes(), "index") {
        Some(value_) => value_,
        None => {
            return ResponseCreator::create_bad_request();
        }
    };

    let mut pooled_connection = match connection_pool.get().await {
        Ok(pooled_connection_) => pooled_connection_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    let connection = &mut *pooled_connection;

    let key_event = StorageKeyManager::create_key_event(
        StorageKeyManager::EVENT_TOKEN_MINTED_PART,
        index_value,
    );

    let data = match connection
        .get::<'_, &'_ str, Option<String>>(&key_event)
        .await
    {
        Ok(data_) => data_,
        Err(_) => {
            return ResponseCreator::create_internal_server_error();
        }
    };

    match data {
        Some(data_) => {
            return ResponseCreator::create_ok(Some(data_));
        }
        None => {
            return ResponseCreator::create_ok(Some(format!(
                "There are no event of TokenMinted type with index {}.",
                index_value
            )))
        }
    }
}

#[derive(Debug)]
struct LogicError;

impl Display for LogicError {
    fn fmt<'a>(&'a self, formatter: &'a mut Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "Logic error.")
    }
}

impl Error for LogicError {}
