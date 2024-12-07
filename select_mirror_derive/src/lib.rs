use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(SelectMirror)]
pub fn process_macro_derive(input: TokenStream) -> TokenStream {
    // 基于 input 构建 AST 语法树
    let ast: DeriveInput = syn::parse(input).unwrap();
    // 构建特征实现代码
    impl_process_macro(&ast)
}

fn impl_process_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        use crate::command::SelectMirror;
        use dialoguer::theme::ColorfulTheme;
        use dialoguer::Select;

        impl SelectMirror for #name {
            fn select(&self) {
                let mirrors = &self.get_mirrors();
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Pick Mirror You Want ")
                    .default(0)
                    .items(mirrors)
                    .interact()
                    .unwrap();
                let mirror = mirrors[selection].clone();
                self.set_mirror(mirror);
            }
        }
    };
    gen.into()
}
