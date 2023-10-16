use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn item(_: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let mut ast = parse_macro_input!(input_clone as ItemStruct);
    let name = ast.ident.clone();

    let fields = match &mut ast.fields {
        Fields::Named(fields) => &mut fields.named,
        Fields::Unnamed(fields) => &mut fields.unnamed,
        Fields::Unit => panic!("Unit struct is not supported"),
    };

    let struct_item: ItemStruct = parse_quote!(
        pub struct T {
            id: usize,
            app: crate::app::SharedApp,
            enabled: crate::property::BoolProperty,
            width: crate::property::SizeProperty,
            height: crate::property::SizeProperty,
            background: crate::property::ItemProperty,
            foreground: crate::property::ItemProperty,
            layout_params: crate::item::LayoutParams,
            on_click: Option<Box<dyn Fn()>>,
        }
    );

    struct_item.fields.iter().for_each(|field| {
        fields.push(field.clone());
    });


    quote!(
        #ast

        impl #name{

            pub fn id(mut self, name: &str) -> Self{
                self.id = self.app.id(name);
                self
            }

            pub fn enabled(mut self, enabled: impl core::convert::Into<crate::property::BoolProperty>)->Self{
                self.enabled = enabled.into();
                let app=self.app.clone();
                self.enabled.lock().add_value_changed_listener(
                    crate::property::ValueChangedListener::new_without_id(move ||{
                    app.need_redraw();
                }));
                self
            }

            pub fn width(mut self, width: impl core::convert::Into<crate::property::SizeProperty>)->Self{
                self.width = width.into();
                let app=self.app.clone();
                self.width.lock().add_value_changed_listener(
                    crate::property::ValueChangedListener::new_without_id(move ||{
                    app.need_redraw();
                }));
                self
            }

            pub fn height(mut self, height: impl core::convert::Into<crate::property::SizeProperty>)->Self{
                self.height = height.into();
                let app=self.app.clone();
                self.height.lock().add_value_changed_listener(
                    crate::property::ValueChangedListener::new_without_id(move ||{
                    app.need_redraw();
                }));
                self
            }

            pub fn background(mut self, background: impl core::convert::Into<crate::property::ItemProperty>)->Self{
                self.background = background.into();
                let app=self.app.clone();
                self.background.lock().add_value_changed_listener(
                    crate::property::ValueChangedListener::new_without_id(move ||{
                    app.need_redraw();
                }));
                self
            }

            pub fn foreground(mut self, foreground: impl core::convert::Into<crate::property::ItemProperty>)->Self{
                self.foreground = foreground.into();
                let app=self.app.clone();
                self.foreground.lock().add_value_changed_listener(
                    crate::property::ValueChangedListener::new_without_id(move ||{
                    app.need_redraw();
                }));
                self
            }

            pub fn on_click(mut self, on_click: impl Fn() + 'static)->Self{
                self.on_click = Some(Box::new(on_click));
                self
            }
        }

        impl crate::item::ItemTrait for #name{
            fn get_id(&self) -> usize{
                self.id
            }

            fn get_enabled(&self)->crate::property::BoolProperty{
                self.enabled.clone()
            }
            
            fn get_width(&self) -> crate::property::SizeProperty{
                self.width.clone()
            }

            fn get_height(&self) -> crate::property::SizeProperty{
                self.height.clone()
            }

            fn get_layout_params(&self) -> &crate::item::LayoutParams{
                &self.layout_params
            }

            fn get_on_click(&self) -> Option<&Box<dyn Fn() + 'static>>{
                self.on_click.as_ref()
            }
        }

    )
        .into()
}