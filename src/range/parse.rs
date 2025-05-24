use chumsky::{Parser, prelude::*};

use super::{RangeExtremeParseable, Ranges};

pub type Extra<'a> = chumsky::extra::Full<chumsky::error::Rich<'a, char>, (), ()>;

pub fn parser<'a, T>() -> impl Parser<'a, &'a str, Ranges<T>, Extra<'a>>
where
    T: RangeExtremeParseable + 'a,
{
    recursive(|parser| {
        let whitespace = text::whitespace().ignored();

        let operator = just("<=")
            .to(Ranges::to_inclusive as fn(T) -> Ranges<T>)
            .or(just("<").to(Ranges::to as fn(T) -> Ranges<T>))
            .or(just(">=").to(Ranges::from as fn(T) -> Ranges<T>))
            .or(just(">").to(Ranges::from_exclusive as fn(T) -> Ranges<T>))
            .or(just("==").to(Ranges::single as fn(T) -> Ranges<T>))
            .or(just("!=").to(Ranges::except as fn(T) -> Ranges<T>));

        let atom = operator
            .then_ignore(whitespace)
            .then(T::parser())
            .map(|(op, t)| op(t))
            .or(just("-").to(Ranges::EMPTY))
            .or(just("*").to(Ranges::full()))
            .or(parser.delimited_by(just('('), just(')')));

        let negated = just("!")
            .and_is(just("!=").not())
            .then_ignore(whitespace)
            .repeated()
            .count()
            .then(atom)
            .map(|(negs, atom)| if negs % 2 == 0 { atom } else { atom.not() });

        let anded = negated.clone().foldl(
            just("&&")
                .padded_by(whitespace)
                .ignore_then(negated)
                .repeated(),
            |a, b| a.and(&b),
        );

        let orred = anded.clone().foldl(
            just("||")
                .padded_by(whitespace)
                .ignore_then(anded)
                .repeated(),
            |a, b| a.or(&b),
        );

        orred.padded_by(whitespace)
    })
}
