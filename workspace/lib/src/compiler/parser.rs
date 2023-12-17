use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_till, take_until};
use nom::character::complete::{alpha1, alphanumeric1, char, crlf, digit1, multispace0, multispace1};
use nom::combinator::{map, opt, recognize};
use nom::IResult;
use nom::multi::{many0, many1, many_till, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom_locate::position;
use crate::common::error::{ParseError, ScriptError};

use crate::compiler::Span;
use crate::compiler::token::{Token, TokenPosition};
use crate::script_parse_error;

const KEYWORD_IMPORT: &str = "import";
const KEYWORD_FUNCTION: &str = "function";
const KEYWORD_END: &str = "end";
const DOT_OPERATOR: &str = ".";


pub struct ParserResult {
    pub tokens: Vec<Token>,
    pub parser_time: std::time::Duration,
}

pub fn parse_script(input: &str) -> Result<ParserResult, ScriptError> {

    let start_parser_timer = std::time::Instant::now();

    let result = many0(
        delimited(
            multispace0,
            alt((
                parse_import,
                parse_comment,
                parse_function,
                parse_class,
                parse_module,
                parse_enum
            )),
            multispace0,
        )
    )(Span::new(input));

    match result {
        Ok((_, tokens)) => Ok(ParserResult {
            tokens,
            parser_time: start_parser_timer.elapsed(),
        }),
        Err(e) => script_parse_error!(ParseError::UnableToParseTokens)
    }
}

fn parse_import(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            terminated(tag_no_case(KEYWORD_IMPORT), multispace1),
            separated_list1(
                tag(DOT_OPERATOR),
                parse_identifier,
            )
        ),
        |source| Token::Import {
            position: TokenPosition::new(&input),
            source,
        },
    )(input)
}

// parse comment starting with double dash till end of line or end of file
fn parse_comment(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag("--"), multispace0),
            take_till(|c| c == '\n' || c == '\r'),
            opt(crlf),
        ),
        |comment: Span| Token::Comment(comment.to_string()),
    )(input)
}

fn parse_print(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag_no_case("print"), multispace1),
            parse_expression,
            multispace0,
        ),
        |expr| Token::Print {
            position: TokenPosition::new(&input),
            expr: Box::from(expr),
        },
    )(input)
}

fn parse_sleep(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag_no_case("sleep"), multispace1),
            parse_expression,
            multispace0,
        ),
        |expr| Token::Sleep {
            position: TokenPosition::new(&input),
            expr: Box::from(expr),
        },
    )(input)
}

//===========================
// FUNCTIONS

// function name(param1, param2) end
fn parse_function(input: Span) -> IResult<Span, Token> {
    map(
        terminated(
            tuple((
                opt(tuple((tag_no_case("static"), multispace1))),
                preceded(tuple((tag_no_case(KEYWORD_FUNCTION), multispace1)), parse_identifier),
                delimited(multispace0, parse_parameters, multispace0),
                opt(preceded(tuple((tag_no_case("as"), multispace1)), parse_identifier)),
                parse_code_block
            )),
            tag_no_case(KEYWORD_END),
        ),
        |(is_static, func_name, params, as_type, body)| Token::Function {
            position: TokenPosition::new(&input),
            function_name: Box::from(func_name),
            is_static: is_static.is_some(),
            scope: None,
            return_type: as_type.map_or(None, |token| Some(Box::from(token))),
            input: params,
            body,
        },
    )(input)
}

fn parse_lambda(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            tag_no_case(KEYWORD_FUNCTION),
            tuple((
                terminated(parse_parameters, multispace0),
                parse_code_block
            )),
            tag_no_case(KEYWORD_END),
        ),
        |(params, body)| Token::AnonFunction {
            position: TokenPosition::new(&input),
            input: params,
            body,
        },
    )(input)
}

fn parse_parameters(input: Span) -> IResult<Span, Vec<Token>> {
    delimited(
        tag("("),
        separated_list0(
            tuple((multispace0, tag(","), multispace0)),
            map(
                tuple((position, parse_variable_declaration)),
                |(pos, (name, as_type, value))| Token::Variable {
                    position: TokenPosition::new(&pos),
                    name: name.to_string(),
                    as_type,
                    value,
                },
            ),
        ),
        tag(")"),
    )(input)
}

fn parse_function_return(input: Span) -> IResult<Span, Token> {
    preceded(
        tuple((tag_no_case("return"), multispace0)),
        map(
            opt(parse_expression),
            |value| Token::Return {
                position: TokenPosition::new(&input),
                expr: value.map_or(None, |token| Some(Box::from(token))),
            },
        ),
    )(input)
}

fn parse_call_function(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_identifier,
            parse_bracketed_arguments
        )),
        |(name, params)| Token::Call {
            position: TokenPosition::new(&input),
            name: Box::from(name),
            input: params,
        },
    )(input)
}

//===========================
// MODULE

fn parse_module(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag_no_case("module"), multispace0),
            tuple((
                terminated(parse_identifier, multispace0),
                many0(
                    delimited(
                        multispace0,
                        alt((
                            parse_comment,
                            parse_variable,
                            parse_enum,
                            parse_class,
                            parse_constant,
                            parse_module,
                            parse_function
                        )),
                        multispace0,
                    )
                ),
            )),
            tag_no_case(KEYWORD_END)
        ),
        |(name, body)| Token::Module {
            position: TokenPosition::new(&input),
            module_name: Box::from(name),
            body,
        },
    )(input)
}

//===========================
// CLASS

fn parse_class(input: Span) -> IResult<Span, Token> {

    map(
        delimited(
            terminated(tag_no_case("class"), multispace0),
            tuple((
                terminated(parse_identifier, multispace0),
                many0(
                    delimited(
                        multispace0,
                        alt((
                            parse_comment,
                            parse_variable,
                            parse_enum,
                            parse_class_constructor,
                            parse_function
                        )),
                        multispace0,
                    )
                ),
            )),
            tag_no_case(KEYWORD_END)
        ),
        |(name, body)| Token::Class {
            position: TokenPosition::new(&input),
            class_name: Box::from(name),
            body,
        },
    )(input)
}

fn parse_class_constructor(input: Span) -> IResult<Span, Token> {
    map(
        terminated(
            tuple((
                delimited(tuple((tag_no_case("constructor"), multispace0)), parse_parameters, multispace0),
                parse_constructor_code_block
            )),
            tag_no_case(KEYWORD_END),
        ),
        |(params, body)| Token::Constructor {
            position: TokenPosition::new(&input),
            input: params,
            body,
        },
    )(input)
}

fn parse_new_keyword(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            tuple((tag_no_case("new"), multispace1)),
            tuple((
                map(
                    tuple((
                        parse_identifier,
                        many0(preceded(tag(DOT_OPERATOR), parse_identifier))
                    )),
                    |(identifier, items)| {
                        Token::DotChain {
                            position: TokenPosition::new(&input),
                            start: Box::from(identifier),
                            chain: items,
                        }
                    },
                ),
                parse_bracketed_arguments
            ))
        ),
        |(id, args)| Token::NewObject { name: Box::from(id), input: args },
    )(input)
}

fn parse_bracketed_arguments(input: Span) -> IResult<Span, Vec<Token>> {
    delimited(
        tag("("),
        separated_list0(tuple((multispace0, tag(","), multispace0)), parse_expression),
        tag(")"),
    )(input)
}

//===========================
// ENUM

fn parse_enum(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            terminated(tag_no_case("enum"), multispace0),
            tuple((
                parse_identifier,
                many_till(
                    delimited(
                        multispace0,
                        parse_identifier,
                        multispace0,
                    ),
                    tag(KEYWORD_END),
                ),
            )),
        ),
        |(name, (items, _))| {
            Token::Enum {
                position: TokenPosition::new(&input),
                name: name.to_string(),
                items
            }
        },
    )(input)
}

//===========================
// CODE BLOCK

fn parse_then_block_end(input: Span) -> IResult<Span, Vec<Token>> {
    delimited(
        tuple((tag_no_case("then"), multispace0)),
        parse_code_block,
        tuple((multispace0, tag_no_case(KEYWORD_END))),
    )(input)
}

fn parse_do_block_end(input: Span) -> IResult<Span, Vec<Token>> {
    delimited(
        tuple((tag_no_case("do"), multispace0)),
        parse_code_block,
        tuple((multispace0, tag_no_case(KEYWORD_END))),
    )(input)
}

fn parse_code_block(input: Span) -> IResult<Span, Vec<Token>> {
    many1(
        delimited(
            multispace0,
            alt((
                parse_comment,
                parse_variable,
                parse_assignment,
                parse_call_function,
                parse_if_chain,
                parse_match_statement,
                parse_while_loop,
                parse_for_in_loop,
                parse_for_to_step,
                parse_print,
                parse_sleep,
                parse_break,
                parse_continue,
                parse_identifier_chain,
                parse_function_return
            )),
            multispace0)
    )(input)
}

fn parse_constructor_code_block(input: Span) -> IResult<Span, Vec<Token>> {
    many1(
        delimited(
            multispace0,
            alt((
                parse_comment,
                parse_print,
                parse_sleep,
                parse_variable,
                parse_assignment,
                parse_call_function,
                parse_if_chain,
                parse_while_loop,
                parse_for_in_loop,
                parse_for_to_step,
                parse_break,
                parse_continue,
                parse_identifier_chain
            )),
            multispace0)
    )(input)
}

//===========================
// VARIABLES

// const name [as type] = 123
fn parse_constant(input: Span) -> IResult<Span, Token> {
    preceded(
        tuple((tag_no_case("const"), multispace0)),
        map(
            parse_variable_declaration,
            |(name, as_type, value)| Token::Constant { name: name.to_string(), as_type, value: value.unwrap() },
        ),
    )(input)
}

// var name [as Integer] = 123
fn parse_variable(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            tuple((tag_no_case("var"), multispace0)),
            parse_variable_declaration,
        ),
        |(name, as_type, value)| Token::Variable {
            position: TokenPosition::new(&input),
            name: name.to_string(),
            as_type,
            value,
        },
    )(input)
}

fn parse_variable_declaration(input: Span) -> IResult<Span, (Token, Option<String>, Option<Box<Token>>)> {
    tuple((
        parse_identifier,
        opt(delimited(
            multispace0,
            map(
                preceded(terminated(tag_no_case("as"), multispace0), parse_identifier),
                |ident| ident.to_string(),
            ),
            multispace0,
        )),
        map(
            opt(preceded(delimited(multispace0, tag("="), multispace0), parse_expression)),
            |v| {
                match v {
                    Some(token) => Some(Box::from(token)),
                    None => None
                }
            },
        )
    ))(input)
}

// name = 123
fn parse_assignment(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            alt((parse_identifier_chain, parse_identifier)),
            preceded(delimited(multispace0, tag("="), multispace0), parse_expression)
        )),
        |(name, value)| Token::Assign {
            position: TokenPosition::new(&input),
            ident: Box::from(name),
            value: Box::from(value),
        },
    )(input)
}

fn parse_identifier_chain(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_identifier,
            many1(
                alt((
                    map(
                        delimited(char('['), parse_expression, char(']')),
                        |item: Token| Token::CollectionIndex(Box::from(item)),
                    ),
                    preceded(
                        char('.'),
                        alt((
                            parse_call_function,
                            parse_identifier
                        )),
                    )
                ))
            )
        )),
        |(identifier, items)| {
            Token::DotChain {
                position: TokenPosition::new(&input),
                start: Box::from(identifier),
                chain: items,
            }
        },
    )(input)
}

//===========================
// IFS

fn parse_if_chain(input: Span) -> IResult<Span, Token> {
    terminated(
        map(
            tuple((
                map(
                    tuple((
                        position,
                        delimited(
                            terminated(tag_no_case("if"), multispace1),
                            parse_expression,
                            tag_no_case("then"),
                        ),
                        parse_code_block,
                    )),
                    |(pos, cond, block)| Token::If {
                        position: TokenPosition::new(&pos),
                        condition: Box::from(cond),
                        body: block,
                    },
                ),
                many0(map(
                    tuple((
                        position,
                        delimited(
                            terminated(tag_no_case("else if"), multispace1),
                            parse_expression,
                            tag_no_case("then"),
                        ),
                        parse_code_block,
                    )),
                    |(pos, cond, block)| Token::If {
                        position: TokenPosition::new(&pos),
                        condition: Box::from(cond),
                        body: block,
                    },
                )),
                opt(map(
                    separated_pair(
                        position,
                        tuple((tag_no_case("else"), multispace1)),
                        parse_code_block,
                    ),
                    |(pos, block)| Token::Else {
                        position: TokenPosition::new(&pos),
                        body: block,
                    },
                ))
            )),
            |(ifstmt, elseifs, elsestmt)| {
                let mut tokens = vec![ifstmt];

                tokens.extend(elseifs);

                if elsestmt.is_some() {
                    tokens.push(elsestmt.unwrap())
                }

                Token::IfChain {
                    position: TokenPosition::new(&input),
                    chain: tokens,
                }
            },
        ),
        tag_no_case(KEYWORD_END),
    )(input)
}

fn parse_match_statement(input: Span) -> IResult<Span, Token> {
    map(
        terminated(
            tuple((
                preceded(position, terminated(tag_no_case("match"), multispace1)),
                delimited(multispace0, parse_expression, multispace0),
                many0(delimited(multispace0, alt((parse_match_case, parse_default_case)), multispace0))
            )),
            preceded(multispace0, tag_no_case(KEYWORD_END))
        ),
        |(pos, expr, arms)| {
            Token::Match {
                position: TokenPosition::new(&pos),
                expr: Box::from(expr),

                // remove default case
                arms: arms.clone().into_iter().filter(|arm| {
                    match arm {
                        Token::DefaultCase { .. } => false,
                        _ => true
                    }
                }).collect(),

                // find the first default
                default: arms.clone().into_iter().find(|arm| {
                    match arm {
                        Token::DefaultCase { .. } => true,
                        _ => false
                    }
                }).map(|arm| Box::from(arm))
            }
        },
    )(input)
}

fn parse_match_case(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            position,
            delimited(
                terminated(tag_no_case("case"), multispace1),
                parse_expression,
                multispace0
            ),
            parse_then_block_end,
        )),
        |(pos, cond, body)| Token::Case {
            position: TokenPosition::new(&pos),
            condition: Box::from(cond),
            body,
        },
    )(input)
}

fn parse_default_case(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            position,
            preceded(
                tuple((tag_no_case("default"), multispace1)),
                parse_then_block_end
            ),
        )),
        |(pos, block)| Token::DefaultCase {
            position: TokenPosition::new(&pos),
            body: block,
        },
    )(input)
}

//===========================
// LOOPS

// while cond do block end
fn parse_while_loop(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tag_no_case("while"),
                parse_expression,
            ),
            parse_do_block_end
        )),
        |(cond, block)| Token::WhileLoop {
            position: TokenPosition::new(&input),
            condition: Box::from(cond),
            body: block,
        },
    )(input)
}

// ((for) (x in v1) (do)) block end
fn parse_for_in_loop(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tag_no_case("for"),
                delimited(multispace1, parse_identifier, multispace1),
            ),
            preceded(
                tag_no_case("in"),
                parse_expression,
            ),
            parse_do_block_end
        )),
        |(ident, target, block)| Token::ForEach {
            position: TokenPosition::new(&input),
            ident: Box::from(ident),
            collection: Box::from(target),
            body: block,
        },
    )(input)
}

// for, (t = start_at) to target step 1, do
fn parse_for_to_step(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tag_no_case("for"),
                delimited(multispace1, parse_identifier, multispace1),
            ),
            preceded(
                tag("="),
                parse_expression,
            ),
            preceded(
                tag_no_case("to"),
                parse_expression,
            ),
            opt(preceded(
                tag_no_case("step"),
                parse_expression,
            )),
            parse_do_block_end
        )),
        |(ident, start, end, step, body)| Token::ForI {
            position: TokenPosition::new(&input),
            ident: Box::from(ident),
            start: Box::from(start),
            step: match step {
                Some(step) => Box::from(step),
                None => Box::from(Token::Integer(1))
            },
            end: Box::from(end),
            body,
        },
    )(input)
}

fn parse_break(input: Span) -> IResult<Span, Token> {
    map(
        tag_no_case("break"),
        |_| Token::Break {
            position: TokenPosition::new(&input)
        },
    )(input)
}

fn parse_continue(input: Span) -> IResult<Span, Token> {
    map(
        tag_no_case("continue"),
        |_| Token::Continue {
            position: TokenPosition::new(&input)
        },
    )(input)
}

//===========================
// LITERALS

fn parse_literal(input: Span) -> IResult<Span, Token> {
    alt((
        parse_array,
        parse_dictionary,
        parse_float,
        parse_integer,
        parse_boolean,
        parse_string,
        parse_null,
        parse_call_function,
        parse_new_keyword,

        // must be last
        parse_identifier_chain,
        parse_identifier,
    ))(input)
}

fn parse_null(input: Span) -> IResult<Span, Token> {
    map(
        tag_no_case("null"),
        |_| Token::Null,
    )(input)
}

fn parse_array(input: Span) -> IResult<Span, Token> {
    delimited(
        tag("["),
        map(
            separated_list0(tuple((multispace0, tag(","), multispace0)), parse_literal),
            |items| Token::Array(items),
        ),
        tag("]"),
    )(input)
}

fn parse_dictionary(input: Span) -> IResult<Span, Token> {
    delimited(
        tag("{"),
        delimited(
            multispace0,
            map(
                separated_list0(
                    tuple((multispace0, tag(","), multispace0)),
                    parse_dictionary_item,
                ),
                |items| {
                    let mut dict = HashMap::new();
                    for (key, value) in items {
                        dict.insert(key.to_string(), value);
                    }
                    Token::Dictionary(dict)
                },
            ),
            multispace0,
        ),
        tag("}"),
    )(input)
}

fn parse_dictionary_item(input: Span) -> IResult<Span, (Token, Token)> {
    map(
        tuple((
            alt((parse_string, parse_identifier)),
            preceded(
                delimited(multispace0, tag(":"), multispace0),
                parse_literal,
            )
        )),
        |(key, value)| (key, value),
    )(input)
}

fn parse_identifier(input: Span) -> IResult<Span, Token> {
    map(
        recognize(
            tuple((
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            ))
        ),
        |s: Span| Token::Identifier {
            position: TokenPosition::new(&input),
            name: s.fragment().to_string(),
        },
    )(input)
}

// (-)123
fn parse_integer(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            opt(tag("-")),
            digit1,
        )),
        |(sign, digits): (Option<Span>, Span)| {
            let mut s = digits.fragment().to_string();
            if sign.is_some() {
                s.insert(0, '-');
            }
            Token::Integer(s.parse::<i32>().unwrap())
        },
    )(input)
}

// (-)123.456
fn parse_float(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            opt(tag("-")),
            separated_pair(
                digit1,
                tag("."),
                digit1,
            )
        )),
        |(sign, (int, frac)): (Option<Span>, (Span, Span))| {
            let mut s = int.fragment().to_string();
            s.push('.');
            s.push_str(frac.fragment());
            if sign.is_some() {
                s.insert(0, '-');
            }
            Token::Float(s.parse::<f32>().unwrap())
        },
    )(input)
}

fn parse_boolean(input: Span) -> IResult<Span, Token> {
    alt((
        map(tag_no_case("true"), |_| Token::Bool(true)),
        map(tag_no_case("false"), |_| Token::Bool(false))
    ))(input)
}

fn parse_string(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            tag("\""),
            take_until("\""),
            tag("\""),
        ),
        |s: Span| Token::String(s.fragment().to_string()),
    )(input)
}

//===========================
// EXPRESSIONS

fn parse_expression(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_expression_and,
            many0(tuple((tag_no_case("or"), parse_expression_and)))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_expression_and(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_expression_stmt,
            many0(tuple((tag_no_case("and"), parse_expression_stmt)))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_expression_stmt(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            opt(delimited(
                multispace0,
                tag_no_case("not"),
                multispace1,
            )),
            delimited(
                multispace0,
                alt((
                    parse_lambda,
                    parse_new_keyword,
                    parse_expression2,
                    parse_call_function,
                    parse_literal,
                )),
                multispace0,
            )
        )),
        |(not, expr)| {
            if not.is_some() {
                Token::Not(Box::from(expr))
            } else {
                expr
            }
        },
    )(input)
}

fn parse_expression2(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_math_expr,
            many0(tuple((alt((tag(">="), tag("<="), tag("=="), tag("!="), tag("<"), tag(">"))), parse_math_expr)))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_math_expr(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_term,
            many0(tuple((alt((tag("+"), tag("-"))), parse_term)))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_term(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_exponents,
            many0(tuple((alt((tag("/"), tag("*"))), parse_exponents)))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_exponents(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            alt((parse_parentheses, parse_value)),
            many0(tuple((tag("^"), parse_value)))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_value(input: Span) -> IResult<Span, Token> {
    delimited(
        multispace0,
        parse_literal,
        multispace0,
    )(input)
}

fn parse_parentheses(input: Span) -> IResult<Span, Token> {
    delimited(
        multispace0,
        delimited(
            char('('),
            parse_expression,
            char(')'),
        ),
        multispace0,
    )(input)
}

fn parse_expr_tag(expr: Token, rem: Vec<(Span, Token)>) -> Token {
    rem.into_iter().fold(expr, |expr1, (op, expr2)| {
        match *op.fragment() {
            "+" => Token::Add(Box::from(expr1), Box::from(expr2)),
            "-" => Token::Sub(Box::from(expr1), Box::from(expr2)),
            "*" => Token::Mul(Box::from(expr1), Box::from(expr2)),
            "/" => Token::Div(Box::from(expr1), Box::from(expr2)),
            "^" => Token::Pow(Box::from(expr1), Box::from(expr2)),
            "==" => Token::Eq(Box::from(expr1), Box::from(expr2)),
            ">" => Token::Gt(Box::from(expr1), Box::from(expr2)),
            ">=" => Token::Ge(Box::from(expr1), Box::from(expr2)),
            "<" => Token::Lt(Box::from(expr1), Box::from(expr2)),
            "<=" => Token::Le(Box::from(expr1), Box::from(expr2)),
            "!=" => Token::Ne(Box::from(expr1), Box::from(expr2)),
            "and" => Token::And(Box::from(expr1), Box::from(expr2)),
            "or" => Token::Or(Box::from(expr1), Box::from(expr2)),
            _ => unimplemented!(),
        }
    })
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::compiler::parser::*;
    use crate::compiler::token::{Token, TokenPosition};

    #[test]
    fn test_program() {
        let input = r#"

            import enterprise.people

            function main()
                var myc = new MyClass()
            end

            class MyClass
                function main()
                    books.read()
                end
            end
        "#;

        let (_, tokens) = parse_script(input).unwrap();

        assert!(tokens.len() > 0);

    }

    #[test]
    fn test_parse_comment_with_crlf() {
        let (_, token) = super::parse_comment(Span::new("-- this is a comment\r\n")).unwrap();
        assert_eq!(token, Token::Comment(String::from("this is a comment")))
    }

    #[test]
    fn test_parse_comment_till_eof() {
        let (_, token) = parse_comment(Span::new("-- this is a comment")).unwrap();
        assert_eq!(token, Token::Comment(String::from("this is a comment")))
    }

    #[test]
    fn test_parse_function() {
        let input = r#"function sub(a, b) as integer
            return a - b
        end"#;

        let (_, token) = parse_function(Span::new(input)).unwrap();

        assert_eq!(token, Token::Function {
            position: TokenPosition { line: 1, column: 1 },
            function_name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 10 },
                name: String::from("sub"),
            }),
            is_static: false,
            scope: None,
            return_type: Some(Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 23 },
                name: String::from("integer"),
            })),
            input: vec![
                Token::Variable {
                    position: TokenPosition { line: 1, column: 14 },
                    name: String::from("a"),
                    as_type: None,
                    value: None,
                },
                Token::Variable {
                    position: TokenPosition { line: 1, column: 17 },
                    name: String::from("b"),
                    as_type: None,
                    value: None,
                },
            ],
            body: vec![
                Token::Return {
                    position: TokenPosition { line: 2, column: 13 },
                    expr: Some(Box::from(Token::Sub(
                        Box::from(Token::Identifier {
                            position: TokenPosition { line: 2, column: 20 },
                            name: String::from("a"),
                        }),
                        Box::from(Token::Identifier {
                            position: TokenPosition { line: 2, column: 24 },
                            name: String::from("b"),
                        }),
                    ))),
                }
            ],
        })
    }

    #[test]
    fn test_parse_function_with_no_return_type() {
        let input = r#"static function sub(a, b)
            return a
        end"#;

        let (_, token) = parse_function(Span::new(input)).unwrap();

        assert_eq!(token, Token::Function {
            position: TokenPosition { line: 1, column: 1 },
            function_name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 17 },
                name: String::from("sub"),
            }),
            is_static: true,
            scope: None,
            return_type: None,
            input: vec![
                Token::Variable {
                    position: TokenPosition { line: 1, column: 21 },
                    name: String::from("a"),
                    as_type: None,
                    value: None,
                },
                Token::Variable {
                    position: TokenPosition { line: 1, column: 24 },
                    name: String::from("b"),
                    as_type: None,
                    value: None,
                },
            ],
            body: vec![
                Token::Return {
                    position: TokenPosition { line: 2, column: 13 },
                    expr: Some(Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 20 },
                        name: String::from("a"),
                    })),
                }
            ],
        })
    }

    #[test]
    fn test_parse_identifier() {
        let (_, token) = parse_identifier(Span::new("add")).unwrap();
        assert_eq!(token, Token::Identifier {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("add"),
        });
    }

    #[test]
    fn test_parse_integer() {
        let (_, token) = parse_integer(Span::new("123")).unwrap();
        assert_eq!(token, Token::Integer(123));
    }

    #[test]
    fn test_parse_negative_integer() {
        let (_, token) = parse_integer(Span::new("-123")).unwrap();
        assert_eq!(token, Token::Integer(-123));
    }

    #[test]
    fn test_parse_float() {
        let (_, token) = parse_float(Span::new("123.456")).unwrap();
        assert_eq!(token, Token::Float(123.456));
    }

    #[test]
    fn test_parse_negative_float() {
        let (_, token) = parse_float(Span::new("-123.456")).unwrap();
        assert_eq!(token, Token::Float(-123.456));
    }

    #[test]
    fn test_parse_boolean_false() {
        let (_, token) = super::parse_boolean(Span::new("true")).unwrap();
        assert_eq!(token, Token::Bool(true));
    }

    #[test]
    fn test_parse_boolean_true() {
        let (_, token) = super::parse_boolean(Span::new("false")).unwrap();
        assert_eq!(token, Token::Bool(false));
    }

    #[test]
    fn test_parse_array() {
        let (_, token) = parse_array(Span::new("[1, true, 3.4]")).unwrap();

        assert_eq!(token, Token::Array(vec![
            Token::Integer(1),
            Token::Bool(true),
            Token::Float(3.4),
        ]));
    }

    #[test]
    fn test_parse_dictionary() {
        let input = r#"{
            "a": 1,
            "b": [true, false, false, true],
            "c": {
                "d": "hello",
                "e": "world",
                "f": {
                    "g": 1,
                    "h": 2,
                    "i": true
                }
            }
        }"#;

        let (_, token) = parse_dictionary(Span::new(input)).unwrap();

        assert_eq!(token, Token::Dictionary(HashMap::from([
            (String::from("a"), Token::Integer(1)),
            (String::from("b"), Token::Array(vec![
                Token::Bool(true),
                Token::Bool(false),
                Token::Bool(false),
                Token::Bool(true),
            ])),
            (String::from("c"), Token::Dictionary(HashMap::from([
                (String::from("d"), Token::String(String::from("hello"))),
                (String::from("e"), Token::String(String::from("world"))),
                (String::from("f"), Token::Dictionary(HashMap::from([
                    (String::from("g"), Token::Integer(1)),
                    (String::from("h"), Token::Integer(2)),
                    (String::from("i"), Token::Bool(true)),
                ])))
            ])))
        ])));
    }

    #[test]
    fn parse_var_with_no_value() {
        let (_, token) = parse_variable(Span::new("var a")).unwrap();

        assert_eq!(token, Token::Variable {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
            as_type: None,
            value: None,
        }
        );
    }

    #[test]
    fn parse_var_with_value() {
        let (_, token) = parse_variable(Span::new("var a = 123")).unwrap();

        assert_eq!(token, Token::Variable {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
            as_type: None,
            value: Some(Box::from(Token::Integer(123))),
        }
        );
    }

    #[test]
    fn parse_var_with_value_as_integer() {
        let (_, token) = parse_variable(Span::new("var a as Integer = 123")).unwrap();

        assert_eq!(token, Token::Variable {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
            as_type: Some(String::from("Integer")),
            value: Some(Box::from(Token::Integer(123))),
        }
        );
    }


    #[test]
    fn test_parse_add_statement() {
        let (_, tokens) = parse_math_expr(Span::new("12 + 34.7")).unwrap();

        assert_eq!(tokens, Token::Add(
            Box::from(Token::Integer(12)),
            Box::from(Token::Float(34.7)))
        );
    }

    #[test]
    fn test_parse_nested_add_sub_statements() {
        let (_, tokens) = parse_math_expr(Span::new("12 - 34 + 15 - 9")).unwrap();

        assert_eq!(tokens, Token::Sub(
            Box::from(Token::Add(
                Box::from(Token::Sub(
                    Box::from(Token::Integer(12)),
                    Box::from(Token::Integer(34)))),
                Box::from(Token::Integer(15)),
            )),
            Box::from(Token::Integer(9)),
        ));
    }

    #[test]
    fn test_parse_multi_level_expression() {
        let (_, tokens) = parse_expression2(Span::new("1 * 2 + 3 / 4 ^ 6 == 2")).unwrap();

        assert_eq!(tokens, Token::Eq(
            Box::from(Token::Add(
                Box::from(Token::Mul(
                    Box::from(Token::Integer(1)),
                    Box::from(Token::Integer(2)))),
                Box::from(Token::Div(
                    Box::from(Token::Integer(3)),
                    Box::from(Token::Pow(
                        Box::from(Token::Integer(4)),
                        Box::from(Token::Integer(6)),
                    )),
                )),
            )),
            Box::from(Token::Integer(2)),
        ));
    }

    #[test]
    fn test_parse_expression_with_parantheses() {
        let (_, token) = parse_expression2(Span::new("(1 + 2) * 3")).unwrap();

        assert_eq!(token, Token::Mul(
            Box::from(Token::Add(
                Box::from(Token::Integer(1)),
                Box::from(Token::Integer(2)))),
            Box::from(Token::Integer(3)),
        ));
    }

    #[test]
    fn test_parse_expression_equals() {
        let (_, token) = parse_expression2(Span::new("1 == 1")).unwrap();

        assert_eq!(token, Token::Eq(
            Box::from(Token::Integer(1)),
            Box::from(Token::Integer(1)),
        ));
    }

    #[test]
    fn test_function_call_with_no_parameters() {
        let (_, token) = parse_call_function(Span::new("add()")).unwrap();

        assert_eq!(token, Token::Call {
            position: TokenPosition { line: 1, column: 1 },
            name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("add"),
            }),
            input: vec![],
        })
    }

    #[test]
    fn test_function_call_with_parameters() {
        let (_, token) = parse_call_function(Span::new(r#"add(test(true), myfunc() == b, 1, 3.6, 4 > 6, "hello world")"#)).unwrap();

        assert_eq!(token, Token::Call {
            position: TokenPosition { line: 1, column: 1 },
            name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("add"),
            }),
            input: vec![
                Token::Call {
                    position: TokenPosition { line: 1, column: 5 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 5 },
                        name: String::from("test"),
                    }),
                    input: vec![
                        Token::Bool(true)
                    ],
                },
                Token::Eq(
                    Box::from(Token::Call {
                        position: TokenPosition { line: 1, column: 17 },
                        name: Box::from(Token::Identifier {
                            position: TokenPosition { line: 1, column: 17 },
                            name: String::from("myfunc"),
                        }),
                        input: vec![],
                    }),
                    Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 29 },
                        name: String::from("b"),
                    }),
                ),
                Token::Integer(1),
                Token::Float(3.6),
                Token::Gt(
                    Box::from(Token::Integer(4)),
                    Box::from(Token::Integer(6)),
                ),
                Token::String(String::from("hello world")),
            ],
        })
    }

    #[test]
    fn test_parse_lambda() {
        let (_, token) = parse_lambda(Span::new(r#"function(a,b)
            return a / b
        end"#)).unwrap();

        assert_eq!(token, Token::AnonFunction {
            position: TokenPosition { line: 1, column: 1 },
            input: vec![
                Token::Variable {
                    position: TokenPosition { line: 1, column: 10 },
                    name: String::from("a"),
                    as_type: None,
                    value: None,
                },
                Token::Variable {
                    position: TokenPosition { line: 1, column: 12 },
                    name: String::from("b"),
                    as_type: None,
                    value: None,
                },
            ],
            body: vec![
                Token::Return {
                    position: TokenPosition { line: 2, column: 13 },
                    expr: Some(Box::from(Token::Div(
                        Box::from(Token::Identifier {
                            position: TokenPosition { line: 2, column: 20 },
                            name: String::from("a"),
                        }),
                        Box::from(Token::Identifier {
                            position: TokenPosition { line: 2, column: 24 },
                            name: String::from("b"),
                        }),
                    ))),
                }
            ],
        })
    }

    #[test]
    fn test_return_with_value() {
        let (_, tokens) = parse_function_return(Span::new("return 123")).unwrap();
        assert_eq!(tokens, Token::Return {
            position: TokenPosition { line: 1, column: 1 },
            expr: Some(Box::from(Token::Integer(123))),
        });
    }

    #[test]
    fn test_return_with_expression() {
        let (_, tokens) = parse_function_return(Span::new("return 7 * b")).unwrap();
        assert_eq!(tokens, Token::Return {
            position: TokenPosition { line: 1, column: 1 },
            expr: Some(Box::from(Token::Mul(
                Box::from(Token::Integer(7)),
                Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 12 },
                    name: String::from("b"),
                }),
            ))),
        });
    }

    #[test]
    fn test_return_with_no_value() {
        let (_, tokens) = parse_function_return(Span::new("return")).unwrap();
        assert_eq!(tokens, Token::Return { position: TokenPosition { line: 1, column: 1 }, expr: None });
    }

    #[test]
    fn test_parse_if_chain() {
        let (_, tokens) = parse_if_chain(Span::new(r#"if 1 == 2 then
            x = 1
        else if 1 > 3 then
            x = 9
        else
            x = 4
        end"#)).unwrap();

        assert_eq!(tokens,
                   Token::IfChain {
                       position: TokenPosition { line: 1, column: 1 },
                       chain: vec![
                           Token::If {
                               position: TokenPosition { line: 1, column: 1 },
                               condition: Box::from(Token::Eq(
                                   Box::from(Token::Integer(1)),
                                   Box::from(Token::Integer(2)),
                               )),
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 2, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 2, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(1)),
                                   }
                               ],
                           },
                           Token::If {
                               position: TokenPosition { line: 3, column: 9 },
                               condition: Box::from(Token::Gt(
                                   Box::from(Token::Integer(1)),
                                   Box::from(Token::Integer(3)),
                               )),
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 4, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 4, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(9)),
                                   }
                               ],
                           },
                           Token::Else {
                               position: TokenPosition { line: 5, column: 9 },
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 6, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 6, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(4)),
                                   }
                               ],
                           },
                       ],
                   }
        )
    }

    #[test]
    fn test_parse_if_else() {
        let (_, tokens) = parse_if_chain(Span::new(r#"if 1 == 2 then
            x = 8
        else
            x = 5
        end"#)).unwrap();

        assert_eq!(tokens,
                   Token::IfChain {
                       position: TokenPosition { line: 1, column: 1 },
                       chain: vec![
                           Token::If {
                               position: TokenPosition { line: 1, column: 1 },
                               condition: Box::from(Token::Eq(
                                   Box::from(Token::Integer(1)),
                                   Box::from(Token::Integer(2)),
                               )),
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 2, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 2, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(8)),
                                   }
                               ],
                           },
                           Token::Else {
                               position: TokenPosition { line: 3, column: 9 },
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 4, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 4, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(5)),
                                   }
                               ],
                           },
                       ],
                   }
        )
    }

    #[test]
    fn test_parse_if_statement() {
        let (_, tokens) = parse_if_chain(Span::new(r#"if 1 == 1 then
            var x = 1
        end"#)).unwrap();

        assert_eq!(tokens,
                   Token::IfChain {
                       position: TokenPosition { line: 1, column: 1 },
                       chain: vec![
                           Token::If {
                               position: TokenPosition { line: 1, column: 1 },
                               condition: Box::from(Token::Eq(
                                   Box::from(Token::Integer(1)),
                                   Box::from(Token::Integer(1)),
                               )),
                               body: vec![
                                   Token::Variable {
                                       position: TokenPosition { line: 2, column: 13 },
                                       name: String::from("x"),
                                       as_type: None,
                                       value: Some(Box::from(Token::Integer(1))),
                                   }
                               ],
                           },
                       ],
                   }
        )
    }

    #[test]
    fn test_match_statement() {

        let (_, token) = parse_match_statement(Span::new(r#"match a

            case 1 then
                print("one")
            end

            case 2 then
                print("two")
            end

            case 3 then
                print("three")
            end

            default then
                print("other")
            end

        end"#)).unwrap();

        assert_eq!(token, Token::Match {
            position: TokenPosition { line: 1, column: 1 },
            expr: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 7 },
                name: String::from("a"),
            }),
            arms: vec![
                Token::Case {
                    position: TokenPosition { line: 3, column: 13 },
                    condition: Box::from(Token::Integer(1)),
                    body: vec![
                        Token::Call {
                            position: TokenPosition { line: 4, column: 17 },
                            name: Box::from(Token::Identifier {
                                position: TokenPosition { line: 4, column: 17 },
                                name: String::from("print"),
                            }),
                            input: vec![
                                Token::String(String::from("one"))
                            ],
                        }
                    ],
                },
                Token::Case {
                    position: TokenPosition { line: 7, column: 13 },
                    condition: Box::from(Token::Integer(2)),
                    body: vec![
                        Token::Call {
                            position: TokenPosition { line: 8, column: 17 },
                            name: Box::from(Token::Identifier {
                                position: TokenPosition { line: 8, column: 17 },
                                name: String::from("print"),
                            }),
                            input: vec![
                                Token::String(String::from("two"))
                            ],
                        }
                    ],
                },
                Token::Case {
                    position: TokenPosition { line: 11, column: 13 },
                    condition: Box::from(Token::Integer(3)),
                    body: vec![
                        Token::Call {
                            position: TokenPosition { line: 12, column: 17 },
                            name: Box::from(Token::Identifier {
                                position: TokenPosition { line: 12, column: 17 },
                                name: String::from("print"),
                            }),
                            input: vec![
                                Token::String(String::from("three"))
                            ],
                        }
                    ],
                }
            ],
            default: Some(Box::new(Token::DefaultCase {
                position: TokenPosition { line: 15, column: 13 },
                body: vec![
                    Token::Call {
                        position: TokenPosition { line: 16, column: 17 },
                        name: Box::from(Token::Identifier {
                            position: TokenPosition { line: 16, column: 17 },
                            name: String::from("print"),
                        }),
                        input: vec![
                            Token::String(String::from("other"))
                        ],
                    }
                ],
            }))
        })

    }

    #[test]
    fn test_parse_match_case() {
        let (_, token) = parse_match_case(Span::new(r#"case 1 then
            print("one")
        end"#)).unwrap();

        assert_eq!(token, Token::Case {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::from(Token::Integer(1)),
            body: vec![
                Token::Call {
                    position: TokenPosition { line: 2, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 13 },
                        name: String::from("print"),
                    }),
                    input: vec![
                        Token::String(String::from("one"))
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_default_case() {
        let (_, token) = parse_default_case(Span::new(r#"default then
            print("other")
        end"#)).unwrap();

        assert_eq!(token, Token::DefaultCase {
            position: TokenPosition { line: 1, column: 1 },
            body: vec![
                Token::Call {
                    position: TokenPosition { line: 2, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 13 },
                        name: String::from("print"),
                    }),
                    input: vec![
                        Token::String(String::from("other"))
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_while_loop() {
        let (_, token) = parse_while_loop(Span::new(r#"while a > 4 do
            print(a)
        end"#)).unwrap();

        assert_eq!(token, Token::WhileLoop {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::from(Token::Gt(
                Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("a"),
                }),
                Box::from(Token::Integer(4)),
            )),
            body: vec![
                Token::Call {
                    position: TokenPosition { line: 2, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 13 },
                        name: String::from("print"),
                    }),
                    input: vec![
                        Token::Identifier {
                            position: TokenPosition { line: 2, column: 19 },
                            name: String::from("a"),
                        }
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_for_in_loop() {
        let (_, token) = parse_for_in_loop(Span::new(r#"for book in books do
            print(book)
        end"#)).unwrap();

        assert_eq!(token, Token::ForEach {
            position: TokenPosition { line: 1, column: 1 },
            ident: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 5 },
                name: String::from("book"),
            }),
            collection: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 13 },
                name: String::from("books"),
            }),
            body: vec![
                Token::Call {
                    position: TokenPosition { line: 2, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 13 },
                        name: String::from("print"),
                    }),
                    input: vec![
                        Token::Identifier {
                            position: TokenPosition { line: 2, column: 19 },
                            name: String::from("book"),
                        }
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_class_method() {
        let (_, tokens) = parse_identifier_chain(Span::new("items.get_message()")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::Call {
                    position: TokenPosition { line: 1, column: 7 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 7 },
                        name: String::from("get_message"),
                    }),
                    input: vec![],
                }
            ],
        })
    }

    #[test]
    fn test_parse_class_method_with_params() {
        let (_, tokens) = parse_identifier_chain(Span::new("items.get_message(2)")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::Call {
                    position: TokenPosition { line: 1, column: 7 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 7 },
                        name: String::from("get_message"),
                    }),
                    input: vec![
                        Token::Integer(2)
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_class_field() {
        let (_, tokens) = parse_identifier_chain(Span::new("items.length")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("length"),
                }
            ],
        })
    }

    #[test]
    fn test_parse_collection_item_with_number() {
        let (_, tokens) = parse_identifier_chain(Span::new("items[6]")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::CollectionIndex(Box::from(Token::Integer(6)))
            ],
        })
    }

    #[test]
    fn test_parse_collection_item_with_string() {
        let (_, tokens) = parse_identifier_chain(Span::new(r#"items["book"]"#)).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::CollectionIndex(Box::from(Token::String(String::from("book"))))
            ],
        })
    }

    #[test]
    fn test_parse_call_chain() {
        let (_, tokens) = parse_identifier_chain(Span::new(r#"items[0][3].get_others()[3]["name"].message"#)).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::CollectionIndex(Box::from(Token::Integer(0))),
                Token::CollectionIndex(Box::from(Token::Integer(3))),
                Token::Call {
                    position: TokenPosition { line: 1, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 13 },
                        name: String::from("get_others"),
                    }),
                    input: vec![],
                },
                Token::CollectionIndex(Box::from(Token::Integer(3))),
                Token::CollectionIndex(Box::from(Token::String(String::from("name")))),
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 37 },
                    name: String::from("message"),
                },
            ],
        });
    }

    #[test]
    fn test_parse_class() {
        let (_, tokens) = parse_class(Span::new(r#"class Book

            var name
            var pages as Integer = 4
            var author as String = "John"

        end"#)).unwrap();

        assert_eq!(tokens, Token::Class {
            position: TokenPosition { line: 1, column: 1 },
            class_name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 7 },
                name: String::from("Book"),
            }),
            body: vec![
                Token::Variable {
                    position: TokenPosition { line: 3, column: 13 },
                    name: String::from("name"),
                    as_type: None,
                    value: None,
                },
                Token::Variable {
                    position: TokenPosition { line: 4, column: 13 },
                    name: String::from("pages"),
                    as_type: Some(String::from("Integer")),
                    value: Some(Box::from(Token::Integer(4))),
                },
                Token::Variable {
                    position: TokenPosition { line: 5, column: 13 },
                    name: String::from("author"),
                    as_type: Some(String::from("String")),
                    value: Some(Box::from(Token::String(String::from("John")))),
                },
            ],
        })
    }

    #[test]
    fn test_new_cass_construct_with_parameters() {
        let (_, tokens) = parse_new_keyword(Span::new(r#"new Book("hello", 123)"#)).unwrap();

        assert_eq!(tokens, Token::NewObject {
            name: Box::from(Token::DotChain {
                position: TokenPosition { line: 1, column: 1 },
                start: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("Book"),
                }),
                chain: vec![],
            }),
            input: vec![
                Token::String(String::from("hello")),
                Token::Integer(123),
            ],
        })
    }

    #[test]
    fn test_new_cass_construct_from_module() {
        let (_, tokens) = parse_new_keyword(Span::new(r#"new Math.Vector2(12, 64)"#)).unwrap();

        assert_eq!(tokens, Token::NewObject {
            name: Box::from(Token::DotChain {
                position: TokenPosition { line: 1, column: 1 },
                start: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("Math"),
                }),
                chain: vec![
                    Token::Identifier {
                        position: TokenPosition { line: 1, column: 10 },
                        name: String::from("Vector2"),
                    },
                ],
            }),
            input: vec![
                Token::Integer(12),
                Token::Integer(64),
            ],
        })
    }


    #[test]
    fn test_parse_enum() {
        let (_, tokens) = parse_enum(Span::new(r#"enum Book
            Book
            Magazine
            Newspaper
        end"#)).unwrap();

        assert_eq!(tokens, Token::Enum {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("Book"),
            items: vec![
                Token::Identifier {
                    position: TokenPosition { line: 2, column: 13 },
                    name: String::from("Book"),
                },
                Token::Identifier {
                    position: TokenPosition { line: 3, column: 13 },
                    name: String::from("Magazine"),
                },
                Token::Identifier {
                    position: TokenPosition { line: 4, column: 13 },
                    name: String::from("Newspaper"),
                },
            ],
        })
    }

    #[test]
    fn test_import_statement() {
        let (_, tokens) = parse_import(Span::new(r#"import graphics.vector2"#)).unwrap();

        assert_eq!(tokens, Token::Import {
            position: TokenPosition { line: 1, column: 1 },
            source: vec![
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 8 },
                    name: String::from("graphics"),
                },
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 17 },
                    name: String::from("vector2"),
                }
            ]
        })
    }

    #[test]
    fn test_import_statement_single_identifier() {
        let (_, tokens) = parse_import(Span::new(r#"import graphics"#)).unwrap();

        assert_eq!(tokens, Token::Import {
            position: TokenPosition { line: 1, column: 1 },
            source: vec![
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 8 },
                    name: String::from("graphics"),
                },
            ]
        })
    }
}
