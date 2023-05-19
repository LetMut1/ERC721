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

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::Version;
use hyper::Body;
use hyper::Response;
use hyper::StatusCode;
use redis::ConnectionInfo;
use std::convert::From;
use std::error::Error;
use std::str::FromStr;
use web3::api::SubscriptionStream;
use web3::transports::WebSocket;
use web3::types::Address;
use web3::types::FilterBuilder;
use web3::types::Log;
use web3::types::H256;
use web3::Web3;

pub use bb8;
pub use bb8_redis;
pub use http;
pub use hyper;
pub use redis;
pub use web3;

pub struct ConnectionPoolCreator;

impl ConnectionPoolCreator {
    const REDIS_URL: &'static str = "redis://redis";

    pub async fn create(
        connection_quantity: u32,
    ) -> Result<Pool<RedisConnectionManager>, Box<dyn Error + 'static>> {
        let connection_info = match ConnectionInfo::from_str(Self::REDIS_URL) {
            Ok(connection_info) => connection_info,
            Err(error) => {
                return Err(Box::from(error));
            }
        };

        let connection_manager = match RedisConnectionManager::new(connection_info) {
            Ok(connection_manager_) => connection_manager_,
            Err(error) => {
                return Err(Box::from(error));
            }
        };

        let connection_pool = match Pool::builder()
            .max_size(connection_quantity)
            .build(connection_manager)
            .await
        {
            Ok(connection_pool_) => connection_pool_,
            Err(error) => {
                return Err(Box::from(error));
            }
        };

        return Ok(connection_pool);
    }
}

pub struct Web3ConnectionCreator;

impl Web3ConnectionCreator {
    const GANACHE_URL: &'static str = "ws://ganache:8545";

    pub async fn create_websocket_connection() -> Result<Web3<WebSocket>, Box<dyn Error + 'static>>
    {
        let transport = match WebSocket::new(Self::GANACHE_URL).await {
            Ok(transport_) => transport_,
            Err(error) => {
                return Err(Box::from(error));
            }
        };

        return Ok(Web3::new(transport));
    }
}

pub struct ContractSubscriptionCreator;

impl ContractSubscriptionCreator {
    pub async fn create<'a>(
        web3: &'a Web3<WebSocket>,
        contract_address: Address,
        event_keccak_256_hash: &'a str,
    ) -> Result<SubscriptionStream<WebSocket, Log>, Box<dyn Error + 'static>> {
        let mut bytes: [u8; 32] = [0; 32];

        if let Err(error) = hex::decode_to_slice(event_keccak_256_hash, bytes.as_mut_slice()) {
            return Err(Box::from(error));
        };

        let hash = H256::from(bytes);

        let filter = FilterBuilder::default()
            .address(vec![contract_address])
            .topics(Some(vec![hash]), None, None, None)
            .build();

        let subscription_stream = match web3.eth_subscribe().subscribe_logs(filter).await {
            Ok(subscription_stream_) => subscription_stream_,
            Err(error) => {
                return Err(Box::from(error));
            }
        };

        return Ok(subscription_stream);
    }
}

pub struct ResponseCreator;

impl ResponseCreator {
    fn create(status_code: StatusCode, data: Option<String>) -> Response<Body> {
        let mut header_map = HeaderMap::new();
        header_map.append(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let mut parts = Response::new(()).into_parts().0;
        parts.status = status_code;
        parts.version = Version::HTTP_2;
        parts.headers = header_map;

        let body = match data {
            Some(data_) => Body::from(data_),
            None => Body::empty(),
        };

        return Response::from_parts(parts, body);
    }

    pub fn create_ok(data: Option<String>) -> Response<Body> {
        return Self::create(StatusCode::OK, data);
    }

    pub fn create_bad_request() -> Response<Body> {
        return Self::create(StatusCode::BAD_REQUEST, None);
    }

    pub fn create_not_found() -> Response<Body> {
        return Self::create(StatusCode::NOT_FOUND, None);
    }

    pub fn create_internal_server_error() -> Response<Body> {
        return Self::create(StatusCode::INTERNAL_SERVER_ERROR, None);
    }
}

pub struct StorageKeyManager;

impl StorageKeyManager {
    pub const EVENT_COLLECTION_CREATED_PART: &'static str = "cc";
    pub const EVENT_COLLECTION_CREATED_QUANTITY: &'static str = "cc:q";
    pub const EVENT_TOKEN_MINTED_PART: &'static str = "tm";
    pub const EVENT_TOKEN_MINTED_QUANTITY: &'static str = "tm:q";
    const SEPARATOR: &'static str = ":";

    pub fn create_key_event<'a>(key_event_part: &'a str, event_index: i64) -> String {
        return format!("{}{}{}", key_event_part, Self::SEPARATOR, event_index);
    }
}
