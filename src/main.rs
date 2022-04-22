extern crate wasmchat;
extern crate yew;
extern crate wasm_logger;

use wasmchat::Model;
use yew::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}