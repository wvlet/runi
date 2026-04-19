//! Implementation of `#[derive(Command)]` for structs and enums.
//
// Clippy on Rust 1.95+ nudges nested `if let` toward let-chains, but we
// still want to compile on older stable rustc where let-chains aren't
// stable, so keep the classic nested form.
#![allow(clippy::collapsible_if, clippy::collapsible_match)]

use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, FieldsUnnamed,
    GenericArgument, Ident, Lit, LitStr, Meta, PathArguments, Result, Type, Variant, parse2,
    spanned::Spanned,
};

pub fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input,
            "#[derive(Command)] is not supported on unions",
        )),
    }
}

// ---------------------------------------------------------------------------
// Struct derive
// ---------------------------------------------------------------------------

fn derive_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let struct_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let cmd_attrs = CommandAttrs::from_attrs(&input.attrs)?;
    let struct_doc = collect_doc(&input.attrs);
    let name = cmd_attrs.name.unwrap_or_else(|| struct_ident.to_string());
    let description = cmd_attrs.description.or(struct_doc).unwrap_or_default();

    let fields = match &data.fields {
        Fields::Named(named) => &named.named,
        Fields::Unit => {
            return Ok(emit_struct_impl(
                input,
                struct_ident,
                impl_generics,
                ty_generics,
                where_clause,
                &name,
                &description,
                &[],
            ));
        }
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                &data.fields,
                "#[derive(Command)] on structs requires named fields",
            ));
        }
    };

    let mut collected: Vec<FieldSpec> = Vec::with_capacity(fields.len());
    for field in fields {
        collected.push(FieldSpec::from_field(field)?);
    }

    Ok(emit_struct_impl(
        input,
        struct_ident,
        impl_generics,
        ty_generics,
        where_clause,
        &name,
        &description,
        &collected,
    ))
}

#[allow(clippy::too_many_arguments)]
fn emit_struct_impl(
    input: &DeriveInput,
    struct_ident: &Ident,
    impl_generics: syn::ImplGenerics,
    ty_generics: syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    name: &str,
    description: &str,
    fields: &[FieldSpec],
) -> TokenStream {
    let _ = input; // reserved for future use
    let schema_stmts = fields.iter().map(FieldSpec::schema_stmt);
    let ctor_stmts = fields.iter().map(FieldSpec::ctor_stmt);

    quote! {
        impl #impl_generics ::runi_cli::Command for #struct_ident #ty_generics #where_clause {
            fn schema() -> ::runi_cli::CommandSchema {
                let schema = ::runi_cli::CommandSchema::new(#name, #description);
                #( let schema = schema #schema_stmts; )*
                schema
            }

            fn from_parsed(
                p: &::runi_cli::ParseResult,
            ) -> ::runi_cli::Result<Self> {
                ::core::result::Result::Ok(Self {
                    #( #ctor_stmts, )*
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Enum derive
// ---------------------------------------------------------------------------

fn derive_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let enum_ident = &input.ident;
    let cmd_attrs = CommandAttrs::from_attrs(&input.attrs)?;
    if cmd_attrs.description.is_some() || cmd_attrs.name.is_some() {
        // Enum-level name/description is accepted but intentionally unused —
        // the registered subcommand names come from each variant. We don't
        // error here so future versions can attach metadata.
    }

    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[derive(Command)] on enums requires at least one variant",
        ));
    }

    let mut variants: Vec<VariantSpec> = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        variants.push(VariantSpec::from_variant(variant)?);
    }

    let registration_body = variants.iter().map(|v| {
        let name_lit = &v.name;
        let inner = &v.inner_ty;
        quote! { .command::<#inner>(#name_lit) }
    });
    let parent_bounds = variants.iter().map(|v| {
        let inner = &v.inner_ty;
        quote! {
            #inner: ::runi_cli::Command + ::runi_cli::SubCommandOf<G> + 'static,
        }
    });

    Ok(quote! {
        impl #enum_ident {
            /// Register each variant as a subcommand on the given launcher.
            ///
            /// The variant's inner type must implement
            /// `::runi_cli::Command` and `::runi_cli::SubCommandOf<G>`.
            pub fn register_on<G>(
                launcher: ::runi_cli::Launcher<G>,
            ) -> ::runi_cli::LauncherWithSubs<G>
            where
                G: ::runi_cli::Command + 'static,
                #( #parent_bounds )*
            {
                launcher #( #registration_body )*
            }
        }
    })
}

struct VariantSpec {
    name: LitStr,
    inner_ty: Type,
}

impl VariantSpec {
    fn from_variant(variant: &Variant) -> Result<Self> {
        let inner_ty = match &variant.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                unnamed.first().unwrap().ty.clone()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &variant.fields,
                    "each variant must wrap exactly one struct — e.g. `Clone(CloneCmd)`",
                ));
            }
        };
        let attrs = CommandAttrs::from_attrs(&variant.attrs)?;
        let name = attrs
            .name
            .map(|s| LitStr::new(&s, variant.ident.span()))
            .unwrap_or_else(|| {
                // default to the variant ident lower-cased
                LitStr::new(
                    &variant.ident.to_string().to_lowercase(),
                    variant.ident.span(),
                )
            });
        Ok(Self { name, inner_ty })
    }
}

// ---------------------------------------------------------------------------
// Attribute parsing
// ---------------------------------------------------------------------------

struct CommandAttrs {
    name: Option<String>,
    description: Option<String>,
}

impl CommandAttrs {
    fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut description = None;
        for attr in attrs {
            if !attr.path().is_ident("command") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(lit_string(&meta.value()?.parse::<Expr>()?)?);
                    Ok(())
                } else if meta.path.is_ident("description") {
                    description = Some(lit_string(&meta.value()?.parse::<Expr>()?)?);
                    Ok(())
                } else {
                    Err(meta.error("unknown key; expected `name` or `description`"))
                }
            })?;
        }
        Ok(Self { name, description })
    }
}

fn lit_string(expr: &Expr) -> Result<String> {
    if let Expr::Lit(syn::ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        Ok(s.value())
    } else {
        Err(syn::Error::new_spanned(expr, "expected a string literal"))
    }
}

/// Collect `#[doc = "..."]` lines into a single trimmed description.
fn collect_doc(attrs: &[Attribute]) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &attr.meta {
            if let Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
            {
                parts.push(s.value().trim().to_string());
            }
        }
    }
    let joined = parts.join(" ").trim().to_string();
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

// ---------------------------------------------------------------------------
// Field analysis
// ---------------------------------------------------------------------------

enum FieldKind {
    Option { prefix: LitStr },
    Argument,
}

enum FieldShape {
    /// `bool` — always a flag.
    Flag,
    /// `Option<T>` — optional value (or optional positional).
    Optional,
    /// `Vec<T>` — repeatable value (option or positional; positional Vec
    /// is not supported in Phase 2).
    Vec,
    /// Any other type — treated as required `T: FromArg`.
    Required,
}

struct FieldSpec {
    ident: Ident,
    ty: Type,
    inner_ty: Type, // The T in Option<T>/Vec<T>, or `ty` itself for direct types.
    shape: FieldShape,
    kind: FieldKind,
    description: String,
}

impl FieldSpec {
    fn from_field(field: &Field) -> Result<Self> {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(field, "named fields are required"))?;
        let ty = field.ty.clone();
        let shape = classify_shape(&ty);
        let inner_ty = inner_type(&ty, &shape);

        let kind = field_kind_from_attrs(&field.attrs)?;

        // Validation: `#[argument]` on a bool is meaningless.
        if matches!(kind, FieldKind::Argument) && matches!(shape, FieldShape::Flag) {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "#[argument] on a bool is not meaningful; use #[option(...)] for flags",
            ));
        }
        // Validation: Vec positional is not supported in Phase 2.
        if matches!(kind, FieldKind::Argument) && matches!(shape, FieldShape::Vec) {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "repeatable positional arguments are not supported; declare a repeatable \
                 option with #[option(\"...\")] on a Vec<T> field instead",
            ));
        }

        let explicit_desc = field_description(&field.attrs)?;
        let doc = collect_doc(&field.attrs);
        let description = explicit_desc.or(doc).unwrap_or_default();

        Ok(Self {
            ident,
            ty,
            inner_ty,
            shape,
            kind,
            description,
        })
    }

    fn argument_name(&self) -> String {
        self.ident.to_string()
    }

    /// Canonical lookup name used when extracting from `ParseResult`.
    /// For options, returns the long (or short) form including dashes so
    /// the emitted code reads naturally at the call site.
    fn lookup_name(&self) -> LitStr {
        match &self.kind {
            FieldKind::Option { prefix } => {
                let (short, long) = split_prefix(&prefix.value());
                let chosen = long.or(short).unwrap_or_default();
                LitStr::new(&chosen, prefix.span())
            }
            FieldKind::Argument => LitStr::new(&self.argument_name(), self.ident.span()),
        }
    }

    /// `.flag(...)`, `.option(...)`, `.argument(...)`, or
    /// `.optional_argument(...)` call suffix that builds the schema.
    fn schema_stmt(&self) -> TokenStream {
        let desc = &self.description;
        match (&self.kind, &self.shape) {
            (FieldKind::Option { prefix }, FieldShape::Flag) => {
                quote_spanned! { prefix.span() => .flag(#prefix, #desc) }
            }
            (FieldKind::Option { prefix }, _) => {
                quote_spanned! { prefix.span() => .option(#prefix, #desc) }
            }
            (FieldKind::Argument, FieldShape::Optional) => {
                let name = self.argument_name();
                quote_spanned! { self.ident.span() => .optional_argument(#name, #desc) }
            }
            (FieldKind::Argument, FieldShape::Vec) => {
                // Repeatable positional isn't supported in Phase 2; surfaced
                // in from_field would be cleaner but we catch it here too
                // defensively.
                let msg = "repeatable positional arguments are not supported; use #[option(...)]";
                quote_spanned! { self.ty.span() => .argument(#msg, #desc) /* unreachable */ }
            }
            (FieldKind::Argument, _) => {
                let name = self.argument_name();
                quote_spanned! { self.ident.span() => .argument(#name, #desc) }
            }
        }
    }

    /// The struct-field initializer for `from_parsed`.
    fn ctor_stmt(&self) -> TokenStream {
        let ident = &self.ident;
        let lookup = self.lookup_name();
        let inner = &self.inner_ty;
        match (&self.kind, &self.shape) {
            (FieldKind::Option { .. }, FieldShape::Flag) => {
                quote_spanned! { ident.span() => #ident: p.flag(#lookup) }
            }
            (FieldKind::Option { .. }, FieldShape::Optional) => {
                quote_spanned! { ident.span() => #ident: p.get::<#inner>(#lookup)? }
            }
            (FieldKind::Option { .. }, FieldShape::Vec) => {
                quote_spanned! { ident.span() => #ident: p.all::<#inner>(#lookup)? }
            }
            (FieldKind::Option { .. }, FieldShape::Required) => {
                quote_spanned! { ident.span() => #ident: p.require::<#inner>(#lookup)? }
            }
            (FieldKind::Argument, FieldShape::Optional) => {
                quote_spanned! { ident.span() => #ident: p.get::<#inner>(#lookup)? }
            }
            (FieldKind::Argument, FieldShape::Vec) => {
                // Rejected at from_field — fall through with a placeholder.
                quote_spanned! { ident.span() => #ident: ::core::default::Default::default() }
            }
            (FieldKind::Argument, FieldShape::Required) => {
                quote_spanned! { ident.span() => #ident: p.require::<#inner>(#lookup)? }
            }
            (FieldKind::Argument, FieldShape::Flag) => {
                // Rejected at from_field — placeholder so the macro output
                // still parses if the rejection somehow slipped through.
                quote_spanned! { ident.span() => #ident: ::core::default::Default::default() }
            }
        }
    }
}

fn field_kind_from_attrs(attrs: &[Attribute]) -> Result<FieldKind> {
    let mut found: Option<FieldKind> = None;
    for attr in attrs {
        if attr.path().is_ident("option") {
            if found.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "field has multiple #[option] / #[argument] attributes",
                ));
            }
            // Parse `"prefix"` or `"prefix", description = "..."`.
            let parser = |stream: syn::parse::ParseStream<'_>| -> Result<LitStr> {
                let prefix: LitStr = stream.parse()?;
                // Drain remaining trailing metadata; description is parsed
                // separately in field_description().
                while !stream.is_empty() {
                    let _: proc_macro2::TokenTree = stream.parse()?;
                }
                Ok(prefix)
            };
            let prefix = attr.parse_args_with(parser)?;
            found = Some(FieldKind::Option { prefix });
        } else if attr.path().is_ident("argument") {
            if found.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "field has multiple #[option] / #[argument] attributes",
                ));
            }
            found = Some(FieldKind::Argument);
        }
    }
    found.ok_or_else(|| {
        syn::Error::new_spanned(
            attrs
                .first()
                .map(|a| a.to_token_stream())
                .unwrap_or_else(|| quote! {}),
            "field is missing #[option(\"...\")] or #[argument] — runi-cli cannot infer intent",
        )
    })
}

/// Optional `description = "..."` inside `#[option(...)]` or `#[argument(...)]`.
fn field_description(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        let is_option = attr.path().is_ident("option");
        let is_argument = attr.path().is_ident("argument");
        if !is_option && !is_argument {
            continue;
        }
        if let Meta::List(list) = &attr.meta {
            let parser = |stream: syn::parse::ParseStream<'_>| -> Result<Option<String>> {
                let mut description: Option<String> = None;
                // Optional leading string literal (the option prefix).
                if stream.peek(LitStr) {
                    let _: LitStr = stream.parse()?;
                    if stream.peek(syn::Token![,]) {
                        let _: syn::Token![,] = stream.parse()?;
                    }
                }
                while !stream.is_empty() {
                    let key: Ident = stream.parse()?;
                    let _: syn::Token![=] = stream.parse()?;
                    let val: LitStr = stream.parse()?;
                    if key == "description" {
                        description = Some(val.value());
                    }
                    if stream.peek(syn::Token![,]) {
                        let _: syn::Token![,] = stream.parse()?;
                    }
                }
                Ok(description)
            };
            let found = list.parse_args_with(parser)?;
            if found.is_some() {
                return Ok(found);
            }
        }
    }
    Ok(None)
}

fn classify_shape(ty: &Type) -> FieldShape {
    if is_path_ident(ty, "bool") {
        return FieldShape::Flag;
    }
    if outer_generic_ident(ty, "Option").is_some() {
        return FieldShape::Optional;
    }
    if outer_generic_ident(ty, "Vec").is_some() {
        return FieldShape::Vec;
    }
    FieldShape::Required
}

fn inner_type(ty: &Type, shape: &FieldShape) -> Type {
    match shape {
        FieldShape::Flag | FieldShape::Required => ty.clone(),
        FieldShape::Optional | FieldShape::Vec => {
            first_generic_arg(ty).unwrap_or_else(|| ty.clone())
        }
    }
}

fn is_path_ident(ty: &Type, target: &str) -> bool {
    if let Type::Path(tp) = ty {
        if tp.qself.is_some() {
            return false;
        }
        if let Some(last) = tp.path.segments.last() {
            return last.ident == target && matches!(last.arguments, PathArguments::None);
        }
    }
    false
}

/// Returns `Some(())` when `ty` is a path ending in `target<_>` (one
/// generic argument).
fn outer_generic_ident(ty: &Type, target: &str) -> Option<()> {
    if let Type::Path(tp) = ty {
        if tp.qself.is_some() {
            return None;
        }
        if let Some(last) = tp.path.segments.last() {
            if last.ident == target {
                if let PathArguments::AngleBracketed(ang) = &last.arguments {
                    if ang.args.len() == 1 {
                        return Some(());
                    }
                }
            }
        }
    }
    None
}

fn first_generic_arg(ty: &Type) -> Option<Type> {
    if let Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if let PathArguments::AngleBracketed(ang) = &last.arguments {
                if let Some(GenericArgument::Type(t)) = ang.args.first() {
                    return Some(t.clone());
                }
            }
        }
    }
    None
}

/// Mirror of schema::split_prefix — parse `"-v,--verbose"` into (short, long).
fn split_prefix(prefix: &str) -> (Option<String>, Option<String>) {
    let mut short = None;
    let mut long = None;
    for part in prefix.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if part.starts_with("--") {
            long = Some(part.to_string());
        } else if part.starts_with('-') {
            short = Some(part.to_string());
        }
    }
    (short, long)
}
