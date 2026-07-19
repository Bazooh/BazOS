use proc_macro::TokenStream;
use quote::quote;
use std::sync::atomic::{AtomicBool, Ordering};
use syn::{ItemFn, parse_macro_input};

static START_DEFINED: AtomicBool = AtomicBool::new(false);

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Enforce uniqueness
    if START_DEFINED.swap(true, Ordering::SeqCst) {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(item),
            "`#[main]` may only be used once",
        )
        .to_compile_error()
        .into();
    }

    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_block = &input.block;

    let mut abi_inputs = Vec::new();
    let mut call_args = Vec::new();
    let mut fn_inputs = Vec::new();

    for arg in &input.sig.inputs {
        match arg {
            syn::FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;

                // Detect &[u64]
                if let syn::Type::Reference(r) = &**ty {
                    if let syn::Type::Slice(slice) = &*r.elem {
                        if let syn::Type::Path(path) = &*slice.elem {
                            if path.path.is_ident("&str") {
                                // ABI: &[&str] -> (*const &str, usize)
                                abi_inputs.push(quote! { ptr: *const &str });
                                abi_inputs.push(quote! { len: usize });

                                fn_inputs.push(quote! { #pat: &[&str] });

                                call_args.push(quote! {
                                    unsafe { core::slice::from_raw_parts(ptr, len) }
                                });

                                continue;
                            }
                        }
                    }
                }

                // default passthrough
                abi_inputs.push(quote! { #pat: #ty });
                fn_inputs.push(quote! { #pat: #ty });
                call_args.push(quote! { #pat });
            }
            _ => {}
        }
    }

    let returns_value = match &input.sig.output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            // Optional: treat explicit -> () as no return value
            !matches!(&**ty, syn::Type::Tuple(t) if t.elems.is_empty())
        }
    };

    let start_body = if returns_value {
        quote! {
            exit(#fn_name(#(#call_args),*))
        }
    } else {
        quote! {
            #fn_name(#(#call_args),*);
            exit(0)
        }
    };

    let expanded = quote! {
        use std::exit::exit;

        #[unsafe(no_mangle)]
        extern "C" fn _start(#(#abi_inputs),*) -> ! {
            #start_body
        }

        fn #fn_name(#(#fn_inputs),*) #fn_block
    };

    expanded.into()
}
