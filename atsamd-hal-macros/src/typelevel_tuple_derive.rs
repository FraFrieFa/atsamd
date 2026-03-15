use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use std::fmt::Write;

struct FieldInfo {
    name: String,
    ty: String,
    state_ident: String,
}

pub fn type_level_tuple(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let (struct_name, fields_group) = parse_struct(&tokens)
        .expect("TypeLevelTuple derive error: expected a struct with named fields");
    let fields = parse_fields(&fields_group);
    let generated = generate_code(&struct_name, &fields);
    generated
        .parse()
        .expect("failed to parse generated TypeLevelTuple code")
}

fn parse_struct(tokens: &[TokenTree]) -> Option<(String, Group)> {
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        if let TokenTree::Ident(ident) = token {
            if ident.to_string() == "struct" {
                let name = match iter.next() {
                    Some(TokenTree::Ident(name)) => name.to_string(),
                    _ => return None,
                };
                while let Some(token) = iter.next() {
                    if let TokenTree::Group(group) = token {
                        if group.delimiter() == Delimiter::Brace {
                            return Some((name, group.clone()));
                        }
                    }
                }
            }
        }
    }
    None
}

fn parse_fields(group: &Group) -> Vec<FieldInfo> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut fields = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                i += 1;
                if let Some(TokenTree::Group(_)) = tokens.get(i) {
                    i += 1;
                }
            }
            TokenTree::Ident(ident) if ident.to_string() == "pub" => {
                i += 1;
                if let Some(TokenTree::Group(group)) = tokens.get(i) {
                    if group.delimiter() == Delimiter::Parenthesis {
                        i += 1;
                    }
                }
            }
            TokenTree::Ident(ident) => {
                let field_name = ident.to_string();
                i += 1;

                if let Some(TokenTree::Punct(p)) = tokens.get(i) {
                    if p.as_char() == ':' {
                        i += 1;
                    }
                }

                let mut ty_tokens = Vec::new();
                let mut angle_depth = 0usize;
                while let Some(token) = tokens.get(i) {
                    match token {
                        TokenTree::Punct(p) if p.as_char() == '<' => {
                            angle_depth += 1;
                            ty_tokens.push(token.clone());
                            i += 1;
                        }
                        TokenTree::Punct(p) if p.as_char() == '>' => {
                            angle_depth = angle_depth.saturating_sub(1);
                            ty_tokens.push(token.clone());
                            i += 1;
                        }
                        TokenTree::Punct(p) if p.as_char() == ',' && angle_depth == 0 => break,
                        TokenTree::Punct(p) if p.as_char() == '#' => break,
                        _ => {
                            ty_tokens.push(token.clone());
                            i += 1;
                        }
                    }
                }

                if let Some(TokenTree::Punct(p)) = tokens.get(i) {
                    if p.as_char() == ',' {
                        i += 1;
                    }
                }

                let ty_stream: TokenStream = ty_tokens.iter().cloned().collect();
                let ty = ty_stream.to_string();
                fields.push(FieldInfo {
                    name: field_name,
                    ty,
                    state_ident: format!("{}State", to_pascal_case(&ident.to_string())),
                });
            }
            TokenTree::Punct(p) if p.as_char() == ',' => {
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    fields
}

fn to_pascal_case(s: &str) -> String {
    s.trim_start_matches("r#")
        .split('_')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn generate_code(struct_name: &str, fields: &[FieldInfo]) -> String {
    let tuple_name = format!("{struct_name}Tuple");
    let state_generics = fields
        .iter()
        .map(|field| format!("{} = crate::typelevel_tuple::Present", field.state_ident))
        .collect::<Vec<_>>()
        .join(", ");

    let field_defs = fields
        .iter()
        .map(|field| {
            format!(
                "    pub {name}: crate::typelevel_tuple::Field<{state}, {ty}>,",
                name = field.name,
                state = field.state_ident,
                ty = field.ty
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let pattern = fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let tuple_field_init = fields
        .iter()
        .map(|field| {
            format!(
                "            {name}: crate::typelevel_tuple::Field::present({name}),",
                name = field.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let struct_field_init = fields
        .iter()
        .map(|field| format!("            {name}: {name}.unwrap(),", name = field.name))
        .collect::<Vec<_>>()
        .join("\n");

    let all_present = fields
        .iter()
        .map(|_| "crate::typelevel_tuple::Present")
        .collect::<Vec<_>>()
        .join(", ");

    let mut output = String::new();
    write!(
        &mut output,
        "\npub struct {tuple}<{state}>{{
{fields}
}}\n",
        tuple = tuple_name,
        state = state_generics,
        fields = field_defs
    )
    .unwrap();

    write!(
        &mut output,
        "\nimpl {name} {{
    pub fn into_tuple(self) -> {tuple} {{
        let {name} {{ {pattern} }} = self;
        {tuple} {{
{inits}
        }}
    }}
}}\n",
        name = struct_name,
        tuple = tuple_name,
        pattern = pattern,
        inits = tuple_field_init
    )
    .unwrap();

    write!(
        &mut output,
        "\nimpl {tuple}<{present}> {{
    pub fn into_struct(self) -> {name} {{
        let {tuple} {{ {pattern} }} = self;
        {name} {{
{inits}
        }}
    }}
}}\n",
        tuple = tuple_name,
        present = all_present,
        name = struct_name,
        pattern = pattern,
        inits = struct_field_init
    )
    .unwrap();

    for index in 0..fields.len() {
        output.push_str(&generate_take_impl(tuple_name.as_str(), fields, index));
        output.push_str(&generate_return_impl(tuple_name.as_str(), fields, index));
    }

    output
}

fn generate_take_impl(tuple_name: &str, fields: &[FieldInfo], target: usize) -> String {
    let field = &fields[target];
    let other_generics: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != target)
        .map(|(_, field)| field.state_ident.clone())
        .collect();

    let generics_decl = if other_generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", other_generics.join(", "))
    };

    let input_states = state_list(fields, target, "crate::typelevel_tuple::Present");
    let output_states = state_list(fields, target, "crate::typelevel_tuple::Absent");

    let pattern = fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let reinit = fields
        .iter()
        .map(|f| {
            if f.name == field.name {
                format!(
                    "            {name}: {name}_absent,",
                    name = f.name
                )
            } else {
                format!("            {name},", name = f.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut block = String::new();
    write!(
        &mut block,
        "\nimpl{generics} {tuple}<{input_states}> {{
    pub fn take_{field}(self) -> ({ty}, {tuple}<{output_states}>) {{
        let {tuple} {{ {pattern} }} = self;
        let (value, {field}_absent) = {field}.take();
        (
            value,
            {tuple} {{
{reinit}
            }},
        )
    }}
}}\n",
        generics = generics_decl,
        tuple = tuple_name,
        input_states = input_states,
        field = field.name,
        ty = field.ty,
        output_states = output_states,
        pattern = pattern,
        reinit = reinit
    )
    .unwrap();

    block
}

fn generate_return_impl(tuple_name: &str, fields: &[FieldInfo], target: usize) -> String {
    let field = &fields[target];
    let other_generics: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != target)
        .map(|(_, field)| field.state_ident.clone())
        .collect();

    let generics_decl = if other_generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", other_generics.join(", "))
    };

    let input_states = state_list(fields, target, "crate::typelevel_tuple::Absent");
    let output_states = state_list(fields, target, "crate::typelevel_tuple::Present");

    let pattern = fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let reinit = fields
        .iter()
        .map(|f| format!("            {name},", name = f.name))
        .collect::<Vec<_>>()
        .join("\n");
    let block = format!(
        "\nimpl{generics} {tuple}<{input_states}> {{
    pub fn return_{field}(self, value: {ty}) -> {tuple}<{output_states}> {{
        let {tuple} {{ {pattern} }} = self;
        let {field} = {field}.insert(value);
        {tuple} {{
{reinit}        }}
    }}
}}\n",
        generics = generics_decl,
        tuple = tuple_name,
        input_states = input_states,
        field = field.name,
        ty = field.ty,
        output_states = output_states,
        pattern = pattern,
        reinit = reinit
    );

    block
}

fn state_list(fields: &[FieldInfo], target: usize, override_state: &str) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            if idx == target {
                override_state.to_string()
            } else {
                field.state_ident.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
