use crate::Errors;
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;
use syn::spanned::Spanned;

pub fn parse_condition(tokens: TokenStream, errors: &Errors) -> TokenStream {
    let condition_span = tokens.span();
    let tokens = collect_tokens(tokens, &errors);

    use ConditionTree::*;
    match &tokens[..] {
        [Value(_), Operator(operator @ (ConditionOperator::And | ConditionOperator::Or), operator_span), Value(_), ..] =>
        {
            let mut values = Vec::with_capacity((tokens.len() + 1) / 2);
            for token in tokens.iter().step_by(2) {
                match token {
                    Value(value) => values.push(value.produce_token_stream(errors)),
                    Operator(_, span) => errors.push_new(span.clone(), "Expected value"),
                }
            }
            for token in tokens.iter().skip(1).step_by(2) {
                if let Value(value) = token {
                    let span = match value {
                        ConditionValue::Literal(literal) => literal.span(),
                        ConditionValue::Group(group) => group.span(),
                        ConditionValue::Column(literal) => literal.span(),
                        ConditionValue::Variable(ident) => ident.span(),
                    };
                    errors.push_new(span, "Unexpected change in operator");
                }
            }
            let variant = Ident::new(
                match operator {
                    ConditionOperator::And => "Conjunction",
                    ConditionOperator::Or => "Disjunction",
                    _ => unreachable!("Should have been checked in match pattern"),
                },
                operator_span.clone(),
            );
            TokenStream::from(quote! {
                ::rorm::conditional::Condition::#variant(
                    vec![
                        #(#values),*
                    ].into_boxed_slice()
                )
            })
        }

        [Operator(operator, span), Value(value)] => {
            let value = value.produce_token_stream(errors);
            let variant = Ident::new(
                match operator {
                    ConditionOperator::Exists => "Exists",
                    ConditionOperator::NotExists => "NotExists",
                    _ => {
                        errors.push_new(span.clone(), "Not a unary operator");
                        return TokenStream::new();
                    }
                },
                span.clone(),
            );
            TokenStream::from(quote! {
                ::rorm::conditional::Condition::UnaryCondition(
                    ::rorm::conditional::UnaryCondition::#variant(Box::new(#value))
                )
            })
        }
        [Value(value), Operator(operator, span)] => {
            let value = value.produce_token_stream(errors);
            let variant = Ident::new(
                match operator {
                    ConditionOperator::IsNull => "IsNull",
                    ConditionOperator::IsNotNull => "IsNotNull",
                    _ => {
                        errors.push_new(span.clone(), "Not a unary operator");
                        return TokenStream::new();
                    }
                },
                span.clone(),
            );
            TokenStream::from(quote! {
                ::rorm::conditional::Condition::UnaryCondition(
                    ::rorm::conditional::UnaryCondition::#variant(Box::new(#value))
                )
            })
        }

        [Value(lhs), Operator(operator, span), Value(rhs)] => {
            let lhs = lhs.produce_token_stream(errors);
            let rhs = rhs.produce_token_stream(errors);
            let variant = Ident::new(
                match operator {
                    ConditionOperator::Equals => "Equals",
                    ConditionOperator::NotEquals => "NotEquals",
                    ConditionOperator::Greater => "Greater",
                    ConditionOperator::GreaterOrEquals => "GreaterOrEquals",
                    ConditionOperator::Less => "Less",
                    ConditionOperator::LessOrEquals => "LessOrEquals",
                    ConditionOperator::Like => "Like",
                    ConditionOperator::NotLike => "NotLike",
                    ConditionOperator::Regexp => "Regexp",
                    ConditionOperator::NotRegexp => "NotRegexp",
                    ConditionOperator::In => "In",
                    ConditionOperator::NotIn => "NotIn",
                    _ => {
                        errors.push_new(span.clone(), "Not a binary operator");
                        return TokenStream::new();
                    }
                },
                span.clone(),
            );
            TokenStream::from(quote! {
                ::rorm::conditional::Condition::BinaryCondition(
                    ::rorm::conditional::BinaryCondition::#variant(Box::new([#lhs, #rhs]))
                )
            })
        }

        [Value(value), Operator(operator @ (ConditionOperator::Between | ConditionOperator::NotBetween), span), Value(lower), Operator(ConditionOperator::And, _), Value(upper)] =>
        {
            let value = value.produce_token_stream(errors);
            let lower = lower.produce_token_stream(errors);
            let upper = upper.produce_token_stream(errors);
            let variant = Ident::new(
                match operator {
                    ConditionOperator::Between => "Between",
                    ConditionOperator::NotBetween => "NotBetween",
                    _ => unreachable!("Should have been checked in match pattern"),
                },
                span.clone(),
            );
            TokenStream::from(quote! {
                ::rorm::conditional::Condition::TernaryCondition(
                    ::rorm::conditional::TernaryCondition::#variant(Box::new([#value, #lower, #upper]))
                )
            })
        }

        _ => {
            errors.push_new(condition_span, "Unknown expression structure");
            return TokenStream::new();
        }
    }
}

#[derive(Eq, PartialEq)]
enum ConditionOperator {
    Equals,
    NotEquals,
    Greater,
    Less,
    GreaterOrEquals,
    LessOrEquals,
    Like,
    NotLike,
    Regexp,
    NotRegexp,
    In,
    NotIn,

    Exists,
    NotExists,
    IsNull,
    IsNotNull,

    Between,
    NotBetween,

    And,
    Or,
}

enum ConditionTree {
    Operator(ConditionOperator, Span),
    Value(ConditionValue),
}

enum ConditionValue {
    Group(Group),
    Column(Literal),
    Literal(Literal),
    Variable(Ident),
}

impl ConditionValue {
    fn produce_token_stream(&self, errors: &Errors) -> TokenStream {
        match self {
            ConditionValue::Group(group) => parse_condition(group.stream(), errors),
            ConditionValue::Literal(literal) => TokenStream::from(quote! {
                ::rorm::conditional::Condition::Value(::rorm::value::Value::from(#literal))
            }),
            ConditionValue::Column(column) => TokenStream::from(quote! {
                ::rorm::conditional::Condition::Value(::rorm::value::Value::Ident(#column))
            }),
            ConditionValue::Variable(variable) => TokenStream::from(quote! {
                ::rorm::conditional::Condition::Value(::rorm::value::Value::from(&#variable))
            }),
        }
    }
}

fn collect_tokens(rust_tokens: TokenStream, errors: &Errors) -> Vec<ConditionTree> {
    let mut tokens = Vec::new();
    let mut prev_punct: Option<Punct> = None;

    let mut is: Option<Span> = None;
    let mut not: Option<Span> = None;

    let mut rust_tokens = rust_tokens.into_iter().peekable();
    while let Some(token) = rust_tokens.next() {
        match token {
            TokenTree::Group(group) => match group.delimiter() {
                Delimiter::Brace => {
                    let mut iterator = group.stream().into_iter();
                    if let Some(TokenTree::Ident(ident)) = iterator.next() {
                        if iterator.next().is_none() {
                            tokens.push(ConditionTree::Value(ConditionValue::Variable(ident)));
                        } else {
                            errors.push_new(group.span(), "Expected a single identifier");
                        }
                    } else {
                        errors.push_new(group.span(), "Expected a single identifier");
                    }
                }
                _ => tokens.push(ConditionTree::Value(ConditionValue::Group(group))),
            },
            TokenTree::Literal(literal) => {
                tokens.push(ConditionTree::Value(ConditionValue::Literal(literal)))
            }
            TokenTree::Punct(punct) => match (punct.spacing(), prev_punct.as_ref()) {
                (Spacing::Alone, None) => {
                    tokens.push(ConditionTree::Operator(
                        match punct.as_char() {
                            '=' => ConditionOperator::Equals,
                            '>' => ConditionOperator::Greater,
                            '<' => ConditionOperator::Less,
                            _ => {
                                errors.push_new(punct.span(), "Unknown comparison");
                                continue;
                            }
                        },
                        punct.span(),
                    ));
                }
                (Spacing::Alone, Some(prev_punct)) => {
                    tokens.push(ConditionTree::Operator(
                        match (prev_punct.as_char(), punct.as_char()) {
                            ('=', '=') => ConditionOperator::Equals,
                            ('<', '>') | ('!', '=') => ConditionOperator::NotEquals,
                            ('>', '=') => ConditionOperator::GreaterOrEquals,
                            ('<', '=') => ConditionOperator::LessOrEquals,
                            _ => {
                                errors.push_new_spanned(
                                    prev_punct.span(),
                                    punct.span(),
                                    "Unknown comparison",
                                );
                                continue;
                            }
                        },
                        join_spans([prev_punct.span(), punct.span()]),
                    ));
                }
                (Spacing::Joint, None) => {
                    prev_punct = Some(punct);
                }
                (Spacing::Joint, Some(prev_punct)) => {
                    let mut spans = Vec::new();
                    spans.push(prev_punct.span());
                    spans.push(punct.span());

                    // consume all joint punct which are still coming
                    while let Some(TokenTree::Punct(next_punct)) =
                        rust_tokens.next_if(|tree| matches!(tree, TokenTree::Punct(_)))
                    {
                        spans.push(next_punct.span());
                        if matches!(next_punct.spacing(), Spacing::Alone) {
                            break;
                        }
                    }

                    errors.push_new_spanned(spans[0], spans[spans.len() - 1], "Unknown comparison");
                }
            },
            TokenTree::Ident(ident) => {
                match (is.as_ref(), not.as_ref(), ident.to_string().as_str()) {
                    (None, None, "IS") => is = Some(ident.span()),
                    (_, None, "NOT") => not = Some(ident.span()),

                    (Some(is_span), Some(not_span), "NULL") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::IsNotNull,
                            join_spans([is_span.clone(), not_span.clone(), ident.span()]),
                        ));
                        is = None;
                        not = None;
                    }
                    (Some(is_span), None, "NULL") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::IsNull,
                            join_spans([is_span.clone(), ident.span()]),
                        ));
                        is = None;
                    }
                    (Some(is_span), _, _) => {
                        errors.push_new_spanned(
                            is_span.clone(),
                            ident.span(),
                            "Expected 'NOT' or 'NULL'",
                        );
                        is = None;
                        not = None;
                    }

                    (None, Some(not_span), "BETWEEN") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::NotBetween,
                            join_spans([not_span.clone(), ident.span()]),
                        ));
                        not = None;
                    }
                    (None, Some(not_span), "LIKE") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::NotLike,
                            join_spans([not_span.clone(), ident.span()]),
                        ));
                        not = None;
                    }
                    (None, Some(not_span), "REGEXP") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::NotRegexp,
                            join_spans([not_span.clone(), ident.span()]),
                        ));
                        not = None;
                    }
                    (None, Some(not_span), "IN") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::NotIn,
                            join_spans([not_span.clone(), ident.span()]),
                        ));
                        not = None;
                    }
                    (None, Some(not_span), "EXISTS") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::NotExists,
                            join_spans([not_span.clone(), ident.span()]),
                        ));
                        not = None;
                    }
                    (None, Some(not_span), _) => {
                        errors.push_new_spanned(
                            not_span.clone(),
                            ident.span(),
                            "Expected one of: 'BETWEEN', 'LIKE', 'REGEXP', 'IN', 'EXISTS'",
                        );
                        not = None;
                    }

                    (None, None, "BETWEEN") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::Between,
                            ident.span(),
                        ));
                    }
                    (None, None, "LIKE") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::Like,
                            ident.span(),
                        ));
                    }
                    (None, None, "REGEXP") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::Regexp,
                            ident.span(),
                        ));
                    }
                    (None, None, "IN") => {
                        tokens.push(ConditionTree::Operator(ConditionOperator::In, ident.span()));
                    }
                    (None, None, "EXISTS") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::Exists,
                            ident.span(),
                        ));
                    }
                    (None, None, "AND") => {
                        tokens.push(ConditionTree::Operator(
                            ConditionOperator::And,
                            ident.span(),
                        ));
                    }
                    (None, None, "OR") => {
                        tokens.push(ConditionTree::Operator(ConditionOperator::Or, ident.span()));
                    }

                    (None, None, _) => {
                        let mut literal = Literal::string(ident.to_string().as_str());
                        literal.set_span(ident.span());
                        tokens.push(ConditionTree::Value(ConditionValue::Column(literal)));
                    }
                }
            }
        }
    }
    return tokens;
}

/// Join an iterator of spans into a single one.
///
/// This only works with a nightly compiler.
/// On stable this will just return the iterator's last span.
///
/// For errors consider [`Errors::push_new_spanned`]
fn join_spans<'a>(spans: impl IntoIterator<Item = Span>) -> Span {
    let mut spans = spans.into_iter();
    let mut span = spans
        .next()
        .expect("`join_spans` shouldn't be called with an empty iterator")
        .clone();
    while let Some(next) = spans.next() {
        span = span.join(next.clone()).unwrap_or(next.clone())
    }
    span
}
