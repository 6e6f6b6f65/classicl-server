/* This file is part of classicl.
 *
 * classicl is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(FixedSize)]
pub fn fixed_size_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        panic!("only packets as structs supported");
    };

    let mut size: usize = 0;

    for i in fields.named.iter() {
        if let syn::Type::Path(p) = &i.ty {
            let t = p.path.segments.last().unwrap().ident.to_string();

            // This implementation is kind of dirty but a little better than the old one i guess
            match &t[..] {
                "Vec" => size += 1024,
                "i8" => size += 1,
                "u8" => size += 1,
                "i16" => size += 2,
                "String" => size += 64,
                _ => {
                    let mut buf = String::new();
                    p.path
                        .segments
                        .iter()
                        .for_each(|x| buf.push_str(&x.ident.to_string()));
                    panic!("{buf} is not supported");
                }
            }
        }
    }

    let gen = quote! {
        impl FixedSize for #name {
            const SIZE: usize = #size;
        }
    };
    gen.into()
}
