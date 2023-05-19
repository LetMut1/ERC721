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
use serde_json::Value;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::str::FromStr;
use web3::contract::Contract;
use web3::contract::Options;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

const COMMAND_CREATE_COLLECTION_ARGUMENT_NAME: &'static str = "name";
const COMMAND_CREATE_COLLECTION_ARGUMENT_SYMBOL: &'static str = "symbol";
const COMMAND_CREATE_COLLECTION: &'static str = "create-collection";
const COMMAND_MINT_ARGUMENT_COLLECTION: &'static str = "collection";
const COMMAND_MINT_ARGUMENT_RECIPIENT: &'static str = "recipient";
const COMMAND_MINT_ARGUMENT_TOKEN_URI: &'static str = "tokenUri";
const COMMAND_MINT: &'static str = "mint";
const CONTRACT_METADATA_FILE_PATH: &'static str =
    "/_task/source/contract/build/contracts/CollectionAggregator.json";
const FROM_ADRESS: &'static str = "from";
const GANACHE_URL: &'static str = "http://ganache:8545";
const TO_ADDRESS: &'static str = "to";

fn main() -> () {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name(COMMAND_CREATE_COLLECTION)
                .about("Create new NFT collection.")
                .arg(
                    Arg::with_name(FROM_ADRESS)
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .help("Sender address."),
                )
                .arg(
                    Arg::with_name(TO_ADDRESS)
                        .index(2)
                        .takes_value(true)
                        .required(true)
                        .help("Called contract address."),
                )
                .arg(
                    Arg::with_name(COMMAND_CREATE_COLLECTION_ARGUMENT_NAME)
                        .index(3)
                        .takes_value(true)
                        .required(true)
                        .help("Collection name."),
                )
                .arg(
                    Arg::with_name(COMMAND_CREATE_COLLECTION_ARGUMENT_SYMBOL)
                        .index(4)
                        .takes_value(true)
                        .required(true)
                        .help("Collection symbol"),
                ),
        )
        .subcommand(
            SubCommand::with_name(COMMAND_MINT)
                .about("Mint NFT into collection.")
                .arg(
                    Arg::with_name(FROM_ADRESS)
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .help("Sender address."),
                )
                .arg(
                    Arg::with_name(TO_ADDRESS)
                        .index(2)
                        .takes_value(true)
                        .required(true)
                        .help("Called contract address."),
                )
                .arg(
                    Arg::with_name(COMMAND_MINT_ARGUMENT_COLLECTION)
                        .index(3)
                        .takes_value(true)
                        .required(true)
                        .help("ERC721 contract address."),
                )
                .arg(
                    Arg::with_name(COMMAND_MINT_ARGUMENT_RECIPIENT)
                        .index(4)
                        .takes_value(true)
                        .required(true)
                        .help("NFT recipient address."),
                )
                .arg(
                    Arg::with_name(COMMAND_MINT_ARGUMENT_TOKEN_URI)
                        .index(5)
                        .takes_value(true)
                        .required(true)
                        .help("Token Uri."),
                ),
        )
        .get_matches();

    let transport = match Http::new(GANACHE_URL) {
        Ok(transport_) => transport_,
        Err(error) => {
            println!("{}", &error);

            return ();
        }
    };

    let web3 = Web3::new(transport);

    let result = match matches.subcommand() {
        (COMMAND_CREATE_COLLECTION, Some(arg_matches)) => {
            let from = value_t_or_exit!(arg_matches, FROM_ADRESS, String);

            let from_ = match Address::from_str(from.as_str()) {
                Ok(to__) => to__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            let to = value_t_or_exit!(arg_matches, TO_ADDRESS, String);

            let to_ = match Address::from_str(to.as_str()) {
                Ok(to__) => to__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            let name =
                value_t_or_exit!(arg_matches, COMMAND_CREATE_COLLECTION_ARGUMENT_NAME, String);

            let symbol = value_t_or_exit!(
                arg_matches,
                COMMAND_CREATE_COLLECTION_ARGUMENT_SYMBOL,
                String
            );

            create_collection(&web3, from_, to_, name, symbol)
        }
        (COMMAND_MINT, Some(arg_matches)) => {
            let from = value_t_or_exit!(arg_matches, FROM_ADRESS, String);

            let from_ = match Address::from_str(from.as_str()) {
                Ok(to__) => to__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            let to = value_t_or_exit!(arg_matches, TO_ADDRESS, String);

            let to_ = match Address::from_str(to.as_str()) {
                Ok(to__) => to__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            let collection =
                value_t_or_exit!(arg_matches, COMMAND_MINT_ARGUMENT_COLLECTION, String);

            let collection_ = match Address::from_str(collection.as_str()) {
                Ok(collection__) => collection__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            let recipient = value_t_or_exit!(arg_matches, COMMAND_MINT_ARGUMENT_RECIPIENT, String);

            let recipient_ = match Address::from_str(recipient.as_str()) {
                Ok(recipient__) => recipient__,
                Err(error) => {
                    println!("{}", &error);

                    return ();
                }
            };

            let token_uri = value_t_or_exit!(arg_matches, COMMAND_MINT_ARGUMENT_TOKEN_URI, String);

            mint(&web3, from_, to_, collection_, recipient_, token_uri)
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
async fn create_collection<'a>(
    web3: &'a Web3<Http>,
    from: Address,
    to: Address,
    name: String,
    symbol: String,
) -> Result<(), Box<dyn Error + 'static>> {
    let contract_abi = get_contract_abi(CONTRACT_METADATA_FILE_PATH)?;

    let contract = match Contract::from_json(web3.eth(), to, contract_abi.as_bytes()) {
        Ok(contract_) => contract_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let options = Options {
        gas: Some(3000000.into()),
        ..Options::default()
    };

    contract
        .call("createCollection", (name, symbol), from, options)
        .await?;

    return Ok(());
}

#[tokio::main]
async fn mint<'a>(
    web3: &'a Web3<Http>,
    from: Address,
    to: Address,
    collection: Address,
    recipient: Address,
    token_uri: String,
) -> Result<(), Box<dyn Error + 'static>> {
    let contract_abi = get_contract_abi(CONTRACT_METADATA_FILE_PATH)?;

    let contract = match Contract::from_json(web3.eth(), to, contract_abi.as_bytes()) {
        Ok(contract_) => contract_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let options = Options {
        gas: Some(150000.into()),
        ..Options::default()
    };

    contract
        .call("mint", (collection, recipient, token_uri), from, options)
        .await?;

    return Ok(());
}

fn get_contract_abi<'a>(file_path: &'a str) -> Result<String, Box<dyn Error + 'static>> {
    let abi_file = match File::open(file_path) {
        Ok(abi_file_) => abi_file_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let json_value = match serde_json::from_reader::<_, Value>(abi_file) {
        Ok(json_file_) => json_file_,
        Err(error) => {
            return Err(Box::from(error));
        }
    };

    let json_value_abi = match json_value.get("abi") {
        Some(json_value_abi_) => json_value_abi_,
        None => {
            return Err(Box::from(LogicError));
        }
    };

    return Ok(json_value_abi.to_string());
}

#[derive(Debug)]
struct LogicError;

impl Display for LogicError {
    fn fmt<'a>(&'a self, formatter: &'a mut Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "Logic error.")
    }
}

impl Error for LogicError {}
