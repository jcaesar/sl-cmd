// Copyright 2019 The Starlark in Rust Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! 90% stolen from the starlark example code.

use std::io::{self, Read};
use std::process::exit;
use std::sync::{Arc, Mutex};

use codemap_diagnostic::{ColorConfig, Emitter};
use starlark::eval::simple::eval;
use starlark::stdlib::global_environment;
use starlark::syntax::dialect::Dialect;

pub fn simple_evaluation(starlark_input: &str, name: &str) -> Result<String, String> {
    let (global_env, type_values) = global_environment();
    global_env.freeze();
    let mut env = global_env.child("simple-cli");

    let map = Arc::new(Mutex::new(codemap::CodeMap::new()));

    let result = eval(
        &map,
        name, 
        &starlark_input,
        Dialect::Bzl,
        &mut env,
        &type_values,
        global_env.clone(),
    );

    match result {
        Ok(res) => Ok(res.to_repr()),
        Err(diagnostic) => {
            let cloned_map_lock = Arc::clone(&map);
            let unlocked_map = cloned_map_lock.lock().unwrap();
            Emitter::stderr(ColorConfig::Always, Some(&unlocked_map)).emit(&[diagnostic]);
            Err(format!("Error interpreting '{}'", starlark_input))
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let mut stdin = String::new();
    let (input, name) = match args.as_slice() {
        [_] => {
            io::stdin()
                .read_to_string(&mut stdin)
                .expect("Error reading from stdin");
            (stdin.as_str(), "[stdin]")
        },
        [_, input] => (input.as_str(), "[arg1]"),
        [] => panic!("0 arguments, command name not set"),
        [cmd, ..] => {
            eprintln!("{}: Provide starlark input either as the only argument, or specify 0 arguments and the code via stdin", cmd);
            exit(3);
        }
    };

    match simple_evaluation(input, name) {
        Ok(result_string) => println!("{}", result_string),
        Err(error_string) => {
            eprintln!("{}", error_string);
            exit(2);
        }
    }
}
