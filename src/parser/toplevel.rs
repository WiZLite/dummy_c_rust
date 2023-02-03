use crate::parser::ty::parse_type;

use super::{statement::parse_statement, token::*, util::*, *};

use nom::{
    branch::permutation,
    character::complete::{multispace0, space0},
    combinator::map,
    error::context,
    multi::{separated_list0},
    sequence::delimited,
};

fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    context(
        "function_decl",
        located(map(
            permutation((
                fn_token,
                delimited(multispace0, parse_identifier, multispace0),
                // params
                delimited(
                    token::lparen,
                    delimited(
                        multispace0,
                        context(
                            "parameters",
                            separated_list0(
                                comma,
                                map(
                                    permutation((
                                        parse_identifier,
                                        skip0,
                                        colon,
                                        skip0,
                                        parse_type,
                                    )),
                                    |(name, _, _, _, ty)| (ty, name),
                                ),
                            ),
                        ),
                        multispace0,
                    ),
                    token::rparen,
                ),
                map(
                    permutation((space0, colon, space0, parse_type)),
                    |(_, _, _, ty)| ty,
                ),
            )),
            |(_, name, params, ty)| FunctionDecl {
                name,
                params,
                return_type: ty,
            },
        )),
    )(input)
}

// pub fn parse_block(input: Span) -> NotLocatedParseResult<Vec<Located<Statement>>> {
//     context(
//         "block",
//         delimited(
//             lbracket,
//             many0(delimited(skip0, parse_statement, skip0)),
//             rbracket,
//         ),
//     )(input)
// }

pub fn parse_block(input: Span) -> NotLocatedParseResult<Vec<Located<Statement>>> {
    context(
        "block",
        delimited(
            lbracket,
            delimited(skip0, separated_list0(skip0, parse_statement), skip0),
            rbracket,
        ),
    )(input)
}

fn parse_function(input: Span) -> ParseResult<TopLevel> {
    located(context(
        "function",
        map(
            permutation((parse_function_decl, skip0, parse_block)),
            |(decl, _, body)| TopLevel::Function {
                decl: decl.value,
                body,
            },
        ),
    ))(input)
}

pub(crate) fn parse_toplevel(input: Span) -> ParseResult<TopLevel> {
    context("toplevel", parse_function)(input)
}
