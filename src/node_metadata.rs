/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

use metadata::{DecodeDifferent, RuntimeMetadata, RuntimeMetadataPrefixed};
use serde::{Deserialize, Serialize};
use parity_codec::alloc::string::FromUtf8Error;

pub fn pretty_format(metadata: &RuntimeMetadataPrefixed) -> Result<String, FromUtf8Error> {
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    metadata.serialize(&mut ser).unwrap();
    String::from_utf8(ser.into_inner())
}

pub type NodeMetadata = Vec<Module>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Module {
    pub name: String,
    pub calls: Vec<Call>,
}

impl Module {
    fn new(name: &DecodeDifferent<&'static str, std::string::String>) -> Module {
        Module { name: format!("{:?}", name).replace("\"", ""), calls: Vec::<Call>::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Call {
    pub name: String,
    pub args: Vec<Arg>,
}

impl Call {
    fn new(name: &DecodeDifferent<&'static str, std::string::String>) -> Call {
        Call { name: format!("{:?}", name).replace("\"", ""), args: Vec::<Arg>::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub ty: String,
}

impl Arg {
    fn new(name: &DecodeDifferent<&'static str, std::string::String>, ty: &DecodeDifferent<&'static str, std::string::String>) -> Arg {
        Arg { name: format!("{:?}", name).replace("\"", ""), ty: format!("{:?}", ty).replace("\"", "") }
    }
}

pub fn parse_metadata_into_module_and_call(metadata: &RuntimeMetadataPrefixed) -> Vec<Module> {
    let mut mod_vec = Vec::<Module>::new();
    match &metadata.1 {
        RuntimeMetadata::V5(value) => {
            match &value.modules {
                DecodeDifferent::Decoded(mods) => {
                    let modules = mods;
                    debug!("-------------------- modules ----------------");
                    for module in modules {
                        debug!("module: {:?}", module.name);
                        let mut _mod = Module::new(&module.name);
                        match &module.calls {
                            Some(DecodeDifferent::Decoded(calls)) => {
                                debug!("-------------------- calls ----------------");

                                if calls.is_empty() {
                                    // indices modules does for some reason list `Some([])' as calls and is thus counted in the call enum
                                    // there might be others doing the same.
                                    _mod.calls.push(Default::default())
                                }

                                for call in calls {
                                    let mut _call = Call::new(&call.name);
                                    match &call.arguments {
                                        DecodeDifferent::Decoded(arguments) => {
                                            for arg in arguments {
                                                _call.args.push(Arg::new(&arg.name, &arg.ty));
                                            }
                                        },
                                        _ => unreachable!("All calls have at least the 'who' argument; qed"),
                                    }
                                    _mod.calls.push(_call);
                                }
                            },
                            _ => debug!("No calls for this module"),
                        }
                        mod_vec.push(_mod);
                    }
                    for m in &mod_vec {
                        info!("{:?}", m);
                    }
                    debug!("successfully decoded metadata");
                },
                _ => unreachable!("There are always modules present; qed"),
            }
        },
        _ => panic!("Unsupported metadata"),
    }
    mod_vec
}