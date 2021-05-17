// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License

use proc_macro2::TokenStream;
use crate::construct_runtime::Pallet;
use syn::{Ident, TypePath};
use quote::{format_ident, quote};

pub fn expand_runtime_metadata(
	runtime: &Ident,
	pallet_declarations: &[Pallet],
	scrate: &TokenStream,
	extrinsic: &TypePath,
) -> TokenStream {
	let modules = pallet_declarations
		.iter()
		.filter_map(|pallet_declaration| {
			pallet_declaration.find_part("Pallet").map(|_| {
				let filtered_names: Vec<_> = pallet_declaration
					.pallet_parts()
					.iter()
					.filter(|part| part.name() != "Pallet")
					.map(|part| part.name())
					.collect();
				(pallet_declaration, filtered_names)
			})
		})
		.map(|(decl, filtered_names)| {
			let name = &decl.name;
			let index = &decl.index;
			let storage = expand_pallet_metadata_storage(&filtered_names, runtime, scrate, decl);
			let calls = expand_pallet_metadata_calls(&filtered_names, runtime, scrate, decl);
			let event = expand_pallet_metadata_events(&filtered_names, runtime, scrate, decl);
			let constants = expand_pallet_metadata_constants(runtime, scrate, decl);
			let errors = expand_pallet_metadata_errors(runtime, scrate, decl);

			quote!{
				#scrate::metadata::ModuleMetadata {
					name: #scrate::metadata::DecodeDifferent::Encode(stringify!(#name)),
					index: #index,
					storage: #storage,
					calls: #calls,
					event: #event,
					constants: #constants,
					errors: #errors,
				}
			}
		})
		.collect::<Vec<_>>();

	quote!{
		impl #runtime {
			pub fn metadata() -> #scrate::metadata::RuntimeMetadataPrefixed {
				#scrate::metadata::RuntimeMetadataLastVersion {
					modules: #scrate::metadata::DecodeDifferent::Encode(&[ #(#modules),* ]),
					extrinsic: #scrate::metadata::ExtrinsicMetadata {
						version: <#extrinsic as #scrate::sp_runtime::traits::ExtrinsicMetadata>::VERSION,
						signed_extensions: <
								<
									#extrinsic as #scrate::sp_runtime::traits::ExtrinsicMetadata
								>::SignedExtensions as #scrate::sp_runtime::traits::SignedExtension
							>::identifier()
								.into_iter()
								.map(#scrate::metadata::DecodeDifferent::Encode)
								.collect(),
					},
				}.into()
			}
		}
	}
}

fn expand_pallet_metadata_storage(
	filtered_names: &[&'static str],
	runtime: &Ident,
	scrate: &TokenStream,
	decl: &Pallet,
) -> TokenStream {
	if filtered_names.contains(&"Storage") {
		let instance = decl.instance.as_ref().into_iter();
		let path = &decl.pallet;

		quote!{
			Some(#scrate::metadata::DecodeDifferent::Encode(
				#scrate::metadata::FnEncode(
					#path::Pallet::<#runtime #(, #path::#instance)*>::storage_metadata
				)
			))
		}
	} else {
		quote!(None)
	}
}

fn expand_pallet_metadata_calls(
	filtered_names: &[&'static str],
	runtime: &Ident,
	scrate: &TokenStream,
	decl: &Pallet,
) -> TokenStream {
	if filtered_names.contains(&"Call") {
		let instance = decl.instance.as_ref().into_iter();
		let path = &decl.pallet;

		quote!{
			Some(#scrate::metadata::DecodeDifferent::Encode(
				#scrate::metadata::FnEncode(
					#path::Pallet::<#runtime #(, #path::#instance)*>::call_functions
				)
			))
		}
	} else {
		quote!(None)
	}
}

fn expand_pallet_metadata_events(
	filtered_names: &[&'static str],
	runtime: &Ident,
	scrate: &TokenStream,
	decl: &Pallet,
) -> TokenStream {
	if filtered_names.contains(&"Event") {
		let mod_name = decl.pallet.mod_name();
		let event = if let Some(instance) = decl.instance.as_ref() {
			format_ident!("__module_events_{}_{}", mod_name, instance)
		} else {
			format_ident!("__module_events_{}", mod_name)
		};

		quote!{
			Some(#scrate::metadata::DecodeDifferent::Encode(
				#scrate::metadata::FnEncode(#runtime::#event)
			))
		}
	} else {
		quote!(None)
	}
}

fn expand_pallet_metadata_constants(
	runtime: &Ident,
	scrate: &TokenStream,
	decl: &Pallet,
) -> TokenStream {
	let path = &decl.pallet;
	let instance = decl.instance.as_ref().into_iter();

	quote!{
		#scrate::metadata::DecodeDifferent::Encode(
			#scrate::metadata::FnEncode(
				#path::Pallet::<#runtime #(, #path::#instance)*>::module_constants_metadata
			)
		)
	}
}

fn expand_pallet_metadata_errors(
	runtime: &Ident,
	scrate: &TokenStream,
	decl: &Pallet,
) -> TokenStream {
	let path = &decl.pallet;
	let instance = decl.instance.as_ref().into_iter();

	quote!{
		#scrate::metadata::DecodeDifferent::Encode(
			#scrate::metadata::FnEncode(
				<#path::Pallet::<#runtime #(, #path::#instance)*> as #scrate::metadata::ModuleErrorMetadata>::metadata
			)
		)
	}
}
