extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Ident, Item};

#[proc_macro_attribute]
pub fn kern_test(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let two: Item = parse(tokens).expect("failed to parse tokens");
    let ret = insert_logging(two);
    ret.into()
}

fn insert_logging(tokens: Item) -> proc_macro2::TokenStream {
    let f = match tokens {
        Item::Fn(f) => f,
        _ => panic!("kern_test can only label fns"),
    };
    let orig = f.sig.ident.clone();
    let name = format!("{}", f.sig.ident.clone());
    let name2 = Ident::new(&format!("{}_", name), proc_macro2::Span::call_site());
    quote! {
        #[test_case]
        fn #name2() {
            #f
            x86_64::instructions::interrupts::without_interrupts(|| {
                serial_print!("running {}... ", #name);
                #orig();
                serial_println!("[ok]");
            });
        }
    }
}
