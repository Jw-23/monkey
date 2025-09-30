use crate::MonkeyError;

pub trait Matcher<I, O> {
    type E: std::error::Error;
    //  主要方法消耗I的所有权
    fn try_parse(&self, input: I) -> Result<(I, O), Self::E>;

    fn into_parse_sequence(self, input: I) -> impl Iterator<Item = Result<O, Self::E>>
    where
        Self: Sized,
    {
        let mut current_input = Some(input);
        let mut has_error = false;
        std::iter::from_fn(move || {
            if has_error {
                return None;
            }
            let inp = current_input.take()?;
            match self.try_parse(inp) {
                Ok((next_inp, out)) => {
                    current_input = Some(next_inp);
                    Some(Ok(out))
                }
                Err(err) => {
                    current_input = None;
                    has_error = true;
                    Some(Err(err))
                }
            }
        })
    }
}

macro_rules! impl_parser_for_tuple {
    ($(($t:ident,$idx:tt)),+ $(,)?) => {
            impl<$($t,)+ I, O> Matcher<I, O> for ($($t,)+)
            where
                $(
                    $t: Fn(I) -> Result<(I, O), MonkeyError>,
                )+
                I:  Clone
            {
                type E = MonkeyError;

                fn try_parse(&self, input: I) -> Result<(I, O), Self::E> {
                    $(
                        match (self.$idx)(input.clone()) {
                            Ok((next_input, out)) => return Ok((next_input, out)),
                            Err(_) => {}
                        };
                    )+
                    return Err(MonkeyError::EOFParserSequence);
                }
            }
    };
}
impl_parser_for_tuple!((F1, 0));
impl_parser_for_tuple!((F1, 0), (F2, 1));
impl_parser_for_tuple!((F1, 0), (F2, 1), (F3, 2));
impl_parser_for_tuple!((F1, 0), (F2, 1), (F3, 2), (F4, 3));
impl_parser_for_tuple!((F1, 0), (F2, 1), (F3, 2), (F4, 3), (F5, 4));
impl_parser_for_tuple!((F1, 0), (F2, 1), (F3, 2), (F4, 3), (F5, 4), (F6, 5));
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6)
);

impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25),
    (F27, 26)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25),
    (F27, 26),
    (F28, 27)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25),
    (F27, 26),
    (F28, 27),
    (F29, 28)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25),
    (F27, 26),
    (F28, 27),
    (F29, 28),
    (F30, 29)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25),
    (F27, 26),
    (F28, 27),
    (F29, 28),
    (F30, 29),
    (F31, 30)
);
impl_parser_for_tuple!(
    (F1, 0),
    (F2, 1),
    (F3, 2),
    (F4, 3),
    (F5, 4),
    (F6, 5),
    (F7, 6),
    (F8, 7),
    (F9, 8),
    (F10, 9),
    (F11, 10),
    (F12, 11),
    (F13, 12),
    (F14, 13),
    (F15, 14),
    (F16, 15),
    (F17, 16),
    (F18, 17),
    (F19, 18),
    (F20, 19),
    (F21, 20),
    (F22, 21),
    (F23, 22),
    (F24, 23),
    (F25, 24),
    (F26, 25),
    (F27, 26),
    (F28, 27),
    (F29, 28),
    (F30, 29),
    (F31, 30),
    (F32, 31)
);

// 从一系列 parser中匹配
pub fn sequence<'a, I, O, P>(
    parser: P,
) -> impl Fn(I) -> Box<dyn Iterator<Item = Result<O, P::E>> + 'a>
where
    I: Clone + 'a,
    O: 'static,
    P: Matcher<I, O> + Clone + 'static,
{
    move |input| {
        // 每次调用闭包时，克隆一次 parser
        let p = parser.clone();
        // 使用 into_... 让返回的 Box 直接拥有这个克隆出来的解析器
        Box::new(p.into_parse_sequence(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug,Clone)]
    struct Input<'a> {
        input: &'a str,
        line: usize,
    }
    #[test]
    fn test_inference() {
        // 尝试不显式指定 N 调用 sequence
        let s = sequence((
            parse_single::<'-'>,
            parse_single::<'+'>,
            parse_single::<'*'>,
        ));
        let mut res = s(Input{ input: "+*-", line: 3});
        assert_eq!(res.next(), Some(Ok('+')));
        assert_eq!(res.next(), Some(Ok('*')));
        assert_eq!(res.next(), Some(Ok('-')));
    }
    fn parse_single<const C: char>(input: Input) -> Result<(Input, char), MonkeyError> {
        if input.input.starts_with(C) {
            let next_i=&input.input[1..];

            return Ok((Input{ input: next_i, line: input.line+1 }, C));
        } else {
            return Err(MonkeyError::ParseError);
        }
    }
}
