//! Parser primitives that accept specific token patterns.
//!
//! *“These creatures you call mice, you see, they are not quite as they appear. They are merely the protrusion into
//! our dimension of vastly hyperintelligent pandimensional beings.”*
//!
//! Chumsky parsers are created by combining together smaller parsers. Right at the bottom of the pile are the parser
//! primitives, a parser developer's bread & butter. Each of these primitives are very easy to understand in isolation,
//! usually only doing one thing.
//!
//! ## The Important Ones
//!
//! - [`just`]: parses a specific input or sequence of inputs
//! - [`filter`]: parses a single input, if the given filter function returns `true`
//! - [`end`]: parses the end of input (i.e: if there any more inputs, this parse fails)

use super::*;

/// See [`end`].
pub struct End<I: ?Sized>(PhantomData<I>);

/// A parser that accepts only the end of input.
///
/// This parser is very useful when you wish to force a parser to consume *all* of the input. It is typically combined
/// with [`Parser::then_ignore`].
///
/// The output type of this parser is `()`.
///
/// # Examples
///
/// ```
/// # use chumsky::prelude::*;
/// assert_eq!(end::<Simple<char>>().parse(""), Ok(()));
/// assert!(end::<Simple<char>>().parse("hello").is_err());
/// ```
///
/// ```
/// # use chumsky::prelude::*;
/// let digits = text::digits::<_, Simple<char>>(10);
///
/// // This parser parses digits!
/// assert_eq!(digits.parse("1234"), Ok("1234".to_string()));
///
/// // However, parsers are lazy and do not consume trailing input.
/// // This can be inconvenient if we want to validate all of the input.
/// assert_eq!(digits.parse("1234AhasjADSJAlaDJKSDAK"), Ok("1234".to_string()));
///
/// // To fix this problem, we require that the end of input follows any successfully parsed input
/// let only_digits = digits.then_ignore(end());
///
/// // Now our parser correctly produces an error if any trailing input is found...
/// assert!(only_digits.parse("1234AhasjADSJAlaDJKSDAK").is_err());
/// // ...while still behaving correctly for inputs that only consist of valid patterns
/// assert_eq!(only_digits.parse("1234"), Ok("1234".to_string()));
/// ```
pub const fn end<I: Input + ?Sized>() -> End<I> {
    End(PhantomData)
}

impl<I: ?Sized> Copy for End<I> {}
impl<I: ?Sized> Clone for End<I> {
    fn clone(&self) -> Self {
        End(PhantomData)
    }
}

impl<'a, I, E, S> Parser<'a, I, (), E, S> for End<I>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, (), E> {
        let before = inp.save();
        match inp.next() {
            (_, None) => Ok(M::bind(|| ())),
            (at, Some(tok)) => Err(Located::at(
                at,
                E::expected_found(None, Some(tok), inp.span_since(before)),
            )),
        }
    }

    go_extra!(());
}

/// See [`empty`].
pub struct Empty<I: ?Sized>(PhantomData<I>);

/// A parser that parses no inputs.
///
/// The output type of this parser is `()`.
pub const fn empty<I: Input + ?Sized>() -> Empty<I> {
    Empty(PhantomData)
}

impl<I: ?Sized> Copy for Empty<I> {}
impl<I: ?Sized> Clone for Empty<I> {
    fn clone(&self) -> Self {
        Empty(PhantomData)
    }
}

impl<'a, I, E, S> Parser<'a, I, (), E, S> for Empty<I>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    fn go<M: Mode>(&self, _: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, (), E> {
        Ok(M::bind(|| ()))
    }

    go_extra!(());
}

pub trait Seq<T> {
    type Iter<'a>: Iterator<Item = T>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

impl<T: Clone> Seq<T> for T {
    type Iter<'a> = core::iter::Once<T>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        core::iter::once(self.clone())
    }
}

impl<'b, T: Clone> Seq<T> for &'b [T] {
    type Iter<'a> = core::iter::Cloned<core::slice::Iter<'a, T>>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        (self as &[T]).iter().cloned()
    }
}

impl<T: Clone, const N: usize> Seq<T> for [T; N] {
    type Iter<'a> = core::array::IntoIter<T, N>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        core::array::IntoIter::new(self.clone())
    }
}

impl<'b, T: Clone, const N: usize> Seq<T> for &'b [T; N] {
    type Iter<'a> = core::array::IntoIter<T, N>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        core::array::IntoIter::new((*self).clone())
    }
}

impl Seq<char> for str {
    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl<'b> Seq<char> for &'b str {
    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl Seq<char> for String {
    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

// impl<'b, T, C: Container<T>> Container<T> for &'b C {
//     type Iter<'a> = C::Iter<'a>;
//     fn iter(&self) -> Self::Iter<'_> { (*self).iter() }
// }

/// See [`just`].
pub struct Just<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for Just<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for Just<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
        }
    }
}

/// A parser that accepts only the given input.
///
/// The output type of this parser is `C`, the input or sequence that was provided.
///
/// # Examples
///
/// ```
/// # use chumsky::{prelude::*, error::Cheap};
/// let question = just::<_, _, Cheap<char>>('?');
///
/// assert_eq!(question.parse("?"), Ok('?'));
/// assert!(question.parse("!").is_err());
/// // This works because parsers do not eagerly consume input, so the '!' is not parsed
/// assert_eq!(question.parse("?!"), Ok('?'));
/// // This fails because the parser expects an end to the input after the '?'
/// assert!(question.then(end()).parse("?!").is_err());
/// ```
pub const fn just<T, I, E, S>(seq: T) -> Just<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    Just {
        seq,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, T> Parser<'a, I, T, E, S> for Just<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, T, E> {
        let mut items = self.seq.iter();
        loop {
            match items.next() {
                Some(next) => {
                    let before = inp.save();
                    match inp.next() {
                        (_, Some(tok)) if next == tok => {}
                        (at, tok) => {
                            break Err(Located::at(
                                at,
                                E::expected_found(Some(Some(next)), tok, inp.span_since(before)),
                            ))
                        }
                    }
                }
                None => break Ok(M::bind(|| self.seq.clone())),
            }
        }
    }

    go_extra!(T);
}

/// See [`one_of`].
pub struct OneOf<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for OneOf<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for OneOf<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
        }
    }
}

/// A parser that accepts one of a sequence of specific inputs.
///
/// The output type of this parser is `I`, the input that was found.
///
/// # Examples
///
/// ```
/// # use chumsky::{prelude::*, error::Cheap};
/// let digits = one_of::<_, _, Cheap<char>>("0123456789")
///     .repeated().at_least(1)
///     .then_ignore(end())
///     .collect::<String>();
///
/// assert_eq!(digits.parse("48791"), Ok("48791".to_string()));
/// assert!(digits.parse("421!53").is_err());
/// ```
pub const fn one_of<T, I, E, S>(seq: T) -> OneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    OneOf {
        seq,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, T> Parser<'a, I, I::Token, E, S> for OneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, I::Token, E> {
        let before = inp.save();
        match inp.next() {
            (_, Some(tok)) if self.seq.iter().any(|not| not == tok) => Ok(M::bind(|| tok)),
            (at, found) => Err(Located::at(
                at,
                E::expected_found(self.seq.iter().map(Some), found, inp.span_since(before)),
            )),
        }
    }

    go_extra!(I::Token);
}

/// See [`none_of`].
pub struct NoneOf<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for NoneOf<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for NoneOf<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
        }
    }
}

/// A parser that accepts any input that is *not* in a sequence of specific inputs.
///
/// The output type of this parser is `I`, the input that was found.
///
/// # Examples
///
/// ```
/// # use chumsky::{prelude::*, error::Cheap};
/// let string = one_of::<_, _, Cheap<char>>("\"'")
///     .ignore_then(none_of("\"'").repeated())
///     .then_ignore(one_of("\"'"))
///     .then_ignore(end())
///     .collect::<String>();
///
/// assert_eq!(string.parse("'hello'"), Ok("hello".to_string()));
/// assert_eq!(string.parse("\"world\""), Ok("world".to_string()));
/// assert!(string.parse("\"421!53").is_err());
/// ```
pub const fn none_of<T, I, E, S>(seq: T) -> NoneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    NoneOf {
        seq,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, T> Parser<'a, I, I::Token, E, S> for NoneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, I::Token, E> {
        let before = inp.save();
        match inp.next() {
            (_, Some(tok)) if self.seq.iter().all(|not| not != tok) => Ok(M::bind(|| tok)),
            (at, found) => Err(Located::at(
                at,
                E::expected_found(None, found, inp.span_since(before)),
            )),
        }
    }

    go_extra!(I::Token);
}

/// See [`any`].
pub struct Any<I: ?Sized, E, S = ()> {
    phantom: PhantomData<(E, S, I)>,
}

impl<I: ?Sized, E, S> Copy for Any<I, E, S> {}
impl<I: ?Sized, E, S> Clone for Any<I, E, S> {
    fn clone(&self) -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<'a, I, E, S> Parser<'a, I, I::Token, E, S> for Any<I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, I::Token, E> {
        let before = inp.save();
        match inp.next() {
            (_, Some(tok)) => Ok(M::bind(|| tok)),
            (at, found) => Err(Located::at(
                at,
                E::expected_found(None, found, inp.span_since(before)),
            )),
        }
    }

    go_extra!(I::Token);
}

/// A parser that accepts any input (but not the end of input).
///
/// The output type of this parser is `I`, the input that was found.
///
/// # Examples
///
/// ```
/// # use chumsky::{prelude::*, error::Cheap};
/// let any = any::<char, Cheap<char>>();
///
/// assert_eq!(any.parse("a"), Ok('a'));
/// assert_eq!(any.parse("7"), Ok('7'));
/// assert_eq!(any.parse("\t"), Ok('\t'));
/// assert!(any.parse("").is_err());
/// ```
pub const fn any<I: Input + ?Sized, E: Error<I>, S>() -> Any<I, E, S> {
    Any {
        phantom: PhantomData,
    }
}

/// See [`take_until`].
pub struct TakeUntil<P, I: ?Sized, OP, C = (), E = (), S = ()> {
    until: P,
    // FIXME try remove OP? See comment in Map declaration
    phantom: PhantomData<(OP, C, E, S, I)>,
}

impl<'a, I, E, S, P, OP, C> TakeUntil<P, OP, I, C, E, S>
where
    I: Input,
    E: Error<I>,
    S: 'a,
    P: Parser<'a, I, OP, E, S>,
{
    pub fn collect<D: Container<OP>>(self) -> TakeUntil<P, OP, D> {
        TakeUntil {
            until: self.until,
            phantom: PhantomData,
        }
    }
}

impl<P: Copy, I: ?Sized, C, E, S> Copy for TakeUntil<P, I, C, E, S> {}
impl<P: Clone, I: ?Sized, C, E, S> Clone for TakeUntil<P, I, C, E, S> {
    fn clone(&self) -> Self {
        TakeUntil {
            until: self.until.clone(),
            phantom: PhantomData,
        }
    }
}

/// A parser that accepts any number of inputs until a terminating pattern is reached.
///
/// The output type of this parser is `(Vec<I>, O)`, a combination of the preceding inputs and the output of the
/// final patterns.
///
/// # Examples
///
/// ```
/// # use chumsky::{prelude::*, error::Cheap};
/// let single_line = just::<_, _, Simple<char>>("//")
///     .then(take_until(text::newline()))
///     .ignored();
///
/// let multi_line = just::<_, _, Simple<char>>("/*")
///     .then(take_until(just("*/")))
///     .ignored();
///
/// let comment = single_line.or(multi_line);
///
/// let tokens = text::ident()
///     .padded()
///     .padded_by(comment
///         .padded()
///         .repeated())
///     .repeated();
///
/// assert_eq!(tokens.parse(r#"
///     // These tokens...
///     these are
///     /*
///         ...have some
///         multi-line...
///     */
///     // ...and single-line...
///     tokens
///     // ...comments between them
/// "#), Ok(vec!["these".to_string(), "are".to_string(), "tokens".to_string()]));
/// ```
pub const fn take_until<'a, P, OP, I, E, S>(until: P) -> TakeUntil<P, I, (), E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    P: Parser<'a, I, OP, E, S>,
{
    TakeUntil {
        until,
        phantom: PhantomData,
    }
}

impl<'a, P, OP, I, E, S, C> Parser<'a, I, (C, OP), E, S> for TakeUntil<P, I, C, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    P: Parser<'a, I, OP, E, S>,
    C: Container<I::Token>,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, (C, OP), E> {
        let mut output = M::bind(|| C::default());

        loop {
            let start = inp.save();
            let e = match self.until.go::<M>(inp) {
                Ok(out) => break Ok(M::combine(output, out, |output, out| (output, out))),
                Err(e) => e,
            };

            inp.rewind(start);

            match inp.next() {
                (_, Some(tok)) => {
                    output = M::map(output, |mut output: C| {
                        output.push(tok);
                        output
                    })
                }
                (_, None) => break Err(e),
            }
        }
    }

    go_extra!((C, OP));
}

/// See [`fn@todo`].
pub struct Todo<I: ?Sized, E>(PhantomData<(E, I)>);

impl<I: ?Sized, E> Copy for Todo<I, E> {}
impl<I: ?Sized, E> Clone for Todo<I, E> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A parser that can be used wherever you need to implement a parser later.
///
/// This parser is analagous to the [`todo!`] and [`unimplemented!`] macros, but will produce a panic when used to
/// parse input, not immediately when invoked.
///
/// This function is useful when developing your parser, allowing you to prototype and run parts of your parser without
/// committing to implementing the entire thing immediately.
///
/// The output type of this parser is whatever you want it to be: it'll never produce output!
///
/// # Examples
///
/// ```should_panic
/// # use chumsky::prelude::*;
/// let int = just::<_, _, Simple<char>>("0x").ignore_then(todo())
///     .or(just("0b").ignore_then(text::digits(2)))
///     .or(text::int(10));
///
/// // Decimal numbers are parsed
/// assert_eq!(int.parse("12"), Ok("12".to_string()));
/// // Binary numbers are parsed
/// assert_eq!(int.parse("0b00101"), Ok("00101".to_string()));
/// // Parsing hexidecimal numbers results in a panic because the parser is unimplemented
/// int.parse("0xd4");
/// ```
pub const fn todo<I: Input + ?Sized, E: Error<I>>() -> Todo<I, E> {
    Todo(PhantomData)
}

impl<'a, I, E, S> Parser<'a, I, (), E, S> for Todo<I, E>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    fn go<M: Mode>(&self, _inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, (), E> {
        todo!("Attempted to use an unimplemented parser")
    }

    go_extra!(());
}

/// See [`choice`].
pub struct Choice<T, O> {
    parsers: T,
    phantom: PhantomData<O>,
}

impl<T: Copy, O> Copy for Choice<T, O> {}
impl<T: Clone, O> Clone for Choice<T, O> {
    fn clone(&self) -> Self {
        Self {
            parsers: self.parsers.clone(),
            phantom: PhantomData,
        }
    }
}

/// Parse using a tuple of many parsers, producing the output of the first to successfully parse.
///
/// This primitive has a twofold improvement over a chain of [`Parser::or`] calls:
///
/// - Rust's trait solver seems to resolve the [`Parser`] impl for this type much faster, significantly reducing
///   compilation times.
///
/// - Parsing is likely a little faster in some cases because the resulting parser is 'less careful' about error
///   routing, and doesn't perform the same fine-grained error prioritisation that [`Parser::or`] does.
///
/// These qualities make this parser ideal for lexers.
///
/// The output type of this parser is the output type of the inner parsers.
///
/// # Examples
/// ```
/// # use chumsky::prelude::*;
/// #[derive(Clone, Debug, PartialEq)]
/// enum Token {
///     If,
///     For,
///     While,
///     Fn,
///     Int(u64),
///     Ident(String),
/// }
///
/// let tokens = choice::<_, Simple<char>>((
///     text::keyword("if").to(Token::If),
///     text::keyword("for").to(Token::For),
///     text::keyword("while").to(Token::While),
///     text::keyword("fn").to(Token::Fn),
///     text::int(10).from_str().unwrapped().map(Token::Int),
///     text::ident().map(Token::Ident),
/// ))
///     .padded()
///     .repeated();
///
/// use Token::*;
/// assert_eq!(
///     tokens.parse("if 56 for foo while 42 fn bar"),
///     Ok(vec![If, Int(56), For, Ident("foo".to_string()), While, Int(42), Fn, Ident("bar".to_string())]),
/// );
/// ```
pub const fn choice<T, O>(parsers: T) -> Choice<T, O> {
    Choice {
        parsers,
        phantom: PhantomData,
    }
}

macro_rules! impl_choice_for_tuple {
    () => {};
    ($head:ident $($X:ident)*) => {
        impl_choice_for_tuple!($($X)*);
        impl_choice_for_tuple!(~ $head $($X)*);
    };
    (~ $($X:ident)*) => {
        #[allow(unused_variables, non_snake_case)]
        impl<'a, I, E, S, $($X),*, O> Parser<'a, I, O, E, S> for Choice<($($X,)*), O>
        where
            I: Input + ?Sized,
            E: Error<I>,
            S: 'a,
            $($X: Parser<'a, I, O, E, S>),*
        {
            fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, O, E> {
                let before = inp.save();

                let Choice { parsers: ($($X,)*), .. } = self;

                let mut err: Option<Located<E>> = None;
                $(
                    match $X.go::<M>(inp) {
                        Ok(out) => return Ok(out),
                        Err(e) => {
                            // TODO: prioritise errors
                            err = Some(match err {
                                Some(err) => err.prioritize(e, |a, b| a.merge(b)),
                                None => e,
                            });
                            inp.rewind(before);
                        },
                    };
                )*

                Err(err.unwrap_or_else(|| Located::at(inp.last_pos(), E::expected_found(None, None, inp.span_since(before)))))
            }

            go_extra!(O);
        }
    };
}

impl_choice_for_tuple!(A_ B_ C_ D_ E_ F_ G_ H_ I_ J_ K_ L_ M_ N_ O_ P_ Q_ S_ T_ U_ V_ W_ X_ Y_ Z_);

#[derive(Copy, Clone)]
pub struct Group<T> {
    parsers: T,
}

pub const fn group<T>(parsers: T) -> Group<T> {
    Group { parsers }
}

macro_rules! flatten_map {
    // map a single element into a 1-tuple
    (<$M:ident> $head:ident) => {
        $M::map(
            $head,
            |$head| ($head,),
        )
    };
    // combine two elements into a 2-tuple
    (<$M:ident> $head1:ident $head2:ident) => {
        $M::combine(
            $head1,
            $head2,
            |$head1, $head2| ($head1, $head2),
        )
    };
    // combine and flatten n-tuples from recursion
    (<$M:ident> $head:ident $($X:ident)+) => {
        $M::combine(
            $head,
            flatten_map!(
                <$M>
                $($X)+
            ),
            |$head, ($($X),+)| ($head, $($X),+),
        )
    };
}

macro_rules! impl_group_for_tuple {
    () => {};
    ($head:ident $ohead:ident $($X:ident $O:ident)*) => {
        impl_group_for_tuple!($($X $O)*);
        impl_group_for_tuple!(~ $head $ohead $($X $O)*);
    };
    (~ $($X:ident $O:ident)*) => {
        #[allow(unused_variables, non_snake_case)]
        impl<'a, I, E, S, $($X),*, $($O),*> Parser<'a, I, ($($O,)*), E, S> for Group<($($X,)*)>
        where
            I: Input + ?Sized,
            E: Error<I>,
            S: 'a,
            $($X: Parser<'a, I, $O, E, S>),*
        {
            fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, ($($O,)*), E> {
                let Group { parsers: ($($X,)*) } = self;

                $(
                    let $X = $X.go::<M>(inp)?;
                )*

                Ok(flatten_map!(<M> $($X)*))
            }

            go_extra!(($($O,)*));
        }
    };
}

impl_group_for_tuple! {
    A_ OA
    B_ OB
    C_ OC
    D_ OD
    E_ OE
    F_ OF
    G_ OG
    H_ OH
    I_ OI
    J_ OJ
    K_ OK
    L_ OL
    M_ OM
    N_ ON
    O_ OO
    P_ OP
    Q_ OQ
    R_ OR
    S_ OS
    T_ OT
    U_ OU
    V_ OV
    W_ OW
    X_ OX
    Y_ OY
    Z_ OZ
}
