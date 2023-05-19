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

use clap::crate_description;
use clap::crate_name;
use clap::crate_version;
use clap::value_t_or_exit;
use clap::App;
use clap::AppSettings;
use clap::Arg;
use clap::SubCommand;
use infrastructure::bb8::Pool;
use infrastructure::bb8_redis::RedisConnectionManager;
use infrastructure::redis::AsyncCommands;
use infrastructure::web3::futures::StreamExt;
use infrastructure::web3::types::Address;
use infrastructure::web3::types::Log;
use infrastructure::ConnectionPoolCreator;
use infrastructure::ContractSubscriptionCreator;
use infrastructure::StorageKeyManager;
use infrastructure::Web3ConnectionCreator;
use std::convert::From;
use std::error::Error;
use std::str::FromStr;

const COMMAND_ARGUMENT_CONTRACT_ADDRESS: &'static str = "contract_address";
const COMMAND_SUBSCRIBE_FOR_COLLECTION_CREATED_EVENT: &'static str =
    "subscribe_for_collection_created_event";
const COMMAND_SUBSCRIBE_FOR_TOKEN_MINTED_EVENT: &'static str = "subscribe_for_token_minted_event";
// keccak_256 hash.
const CONTRACT_EVENT_COLLECTION_CREATED_HASH: &'static str =
    "3454b57f2dca4f5a54e8358d096ac9d1a0d2dab98991ddb89ff9ea1746260617";
// keccak_256 hash.
const CONTRACT_EVENT_TOKEN_MINTED_HASH: &'static str =
    "c9fee7cd4889f66f10ff8117316524260a5242e88e25e0656dfb3f4196a21917";

fn main() -> () {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name(COMMAND_SUBSCRIBE_FOR_COLLECTION_CREATED_EVENT)
                .about("Listen for 'CollectionCreated' event.")
                .arg(
                    Arg::with_name(COMMAND_ARGUMENT_CONTRACT_ADDRESS)
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .help("Contract address."),
                ),
        )
        .subcommand(
            SubCommand::with_name(COMMAND_SUBSCRIBE_FOR_TOKEN_MINTED_EVENT)
                .about("Listen for 'TokenCreated' event.")
                .arg(
                    Arg::with_name(COMMAND_ARGUMENT_CONTRACT_ADDRESS)
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .help("Contract address."),
                ),
        )
        .get_matches();

    let result = match matches.subcommand() {
        (COMMAND_SUBSCRIBE_FOR_COLLECTION_CREATED_EVENT, Some(arg_matches)) => {
            let contract_address =
                value_t_or_exit!(arg_matches, COMMAND_ARGUMENT_CONTRACT_ADDRESS, String);

            let contract_address_ = match Address::from_str(contract_address.as_str()) {
                Ok(contract_address__) => contract_address__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            subscribe(ContractEvent::CollectionCreated, contract_address_)
        }
        (COMMAND_SUBSCRIBE_FOR_TOKEN_MINTED_EVENT, Some(arg_matches)) => {
            let contract_address =
                value_t_or_exit!(arg_matches, COMMAND_ARGUMENT_CONTRACT_ADDRESS, String);

            let contract_address_ = match Address::from_str(contract_address.as_str()) {
                Ok(contract_address__) => contract_address__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            subscribe(ContractEvent::TokenMinted, contract_address_)
        }
        _ => {
            println!("Invalid command");

            return ();
        }
    };

    if let Err(error) = result {
        println!("{}", &error);
    }

    return ();
}

#[tokio::main]
async fn subscribe(
    contract_event: ContractEvent,
    contract_address: Address,
) -> Result<(), Box<dyn Error + 'static>> {
    let connection_pool = ConnectionPoolCreator::create(2).await?;

    let web3 = Web3ConnectionCreator::create_websocket_connection().await?;

    match contract_event {
        ContractEvent::CollectionCreated => {
            let subscription_stream = ContractSubscriptionCreator::create(
                &web3,
                contract_address,
                CONTRACT_EVENT_COLLECTION_CREATED_HASH,
            )
            .await?;

            subscription_stream
                .for_each(|log| async {
                    match log {
                        Ok(log_) => {
                            if let Err(error) = push(
                                &log_,
                                &connection_pool,
                                StorageKeyManager::EVENT_COLLECTION_CREATED_QUANTITY,
                                StorageKeyManager::EVENT_COLLECTION_CREATED_PART,
                            )
                            .await
                            {
                                panic!("{}", error);
                            }
                        }
                        Err(error) => {
                            panic!("{}", error);
                        }
                    }
                })
                .await;
        }
        ContractEvent::TokenMinted => {
            let subscription_stream = ContractSubscriptionCreator::create(
                &web3,
                contract_address,
                CONTRACT_EVENT_TOKEN_MINTED_HASH,
            )
            .await?;

            subscription_stream
                .for_each(|log| async {
                    match log {
                        Ok(log_) => {
                            if let Err(error) = push(
                                &log_,
                                &connection_pool,
                                StorageKeyManager::EVENT_TOKEN_MINTED_QUANTITY,
                                StorageKeyManager::EVENT_TOKEN_MINTED_PART,
                            )
                            .await
                            {
                                panic!("{}", error);
                            }
                        }
                        Err(error) => {
                            panic!("{}", error);
                        }
                    }
                })
                .await;
        }
    }

    return Ok(());
}

async fn push<'a>(
    log: &'a Log,
    connection_pool: &'a Pool<RedisConnectionManager>,
    key_event_quantity: &'a str,
    key_event_part: &'a str,
) -> Result<(), Box<dyn Error + 'static>> {
    let log_ = match serde_json::to_string(log) {
        Ok(data_) => data_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let mut pooled_connection = match connection_pool.get().await {
        Ok(pooled_connection_) => pooled_connection_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let connection = &mut *pooled_connection;

    let row_quantity = match connection
        .incr::<'_, &'_ str, i64, i64>(key_event_quantity, 1)
        .await
    {
        Ok(row_quantity_) => row_quantity_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let key_event = StorageKeyManager::create_key_event(key_event_part, row_quantity);

    if let Err(error) = connection
        .set::<'_, String, String, ()>(key_event, log_)
        .await
    {
        return Err(Box::from(error));
    }

    return Ok(());
}

enum ContractEvent {
    CollectionCreated,
    TokenMinted,
}
