extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// Run a test after an EAL environment was initialized.
///
/// Invoke as `#[rte_test(mock_lcore)]` to mock the current lcore when running the test.
#[proc_macro_attribute]
pub fn rte_test(args: TokenStream, item: TokenStream) -> TokenStream {
    let syn::ItemFn { attrs, vis, sig, block } = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mock_lcore = match &args[..] {
        [] => false,
        [syn::NestedMeta::Meta(syn::Meta::Path(path))]
            if path.get_ident().map(ToString::to_string).as_deref() == Some("mock_lcore") =>
        {
            true
        }
        _ => panic!("Only possible argument to `rte_test` is \"mock_lcore\"."),
    };

    let mock_lcore = mock_lcore.then(|| {
        quote! {
            rte::test_utils::mock_lcore();
        }
    });

    quote! {
        #[test]
        #(#attrs)*
        #vis #sig {
            rte::test_utils::init_test_env();
            #mock_lcore;

            #block;
        }
    }
    .into()
}
