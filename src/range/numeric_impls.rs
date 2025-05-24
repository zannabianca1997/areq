use super::{RangeExtreme, RangeExtremeDisplay, RangeExtremeFromStr};

macro_rules! impl_numeric {
    (
       $( $t:ty )*
    ) => {
        $(
            impl RangeExtreme for $t {
                const MIN: Self = <$t>::MIN;
                const MAX: Self = <$t>::MAX;

                fn next(self) -> Self {
                    self + 1
                }

                fn compare_next_to(&self, other: &Self) -> bool {
                    self.next() == *other
                }
            }

            impl RangeExtremeDisplay for $t {
                fn has_prev(&self) -> bool {
                    self > &<$t>::MIN
                }

                fn display_prev(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self - 1)
                }
            }

            impl RangeExtremeFromStr for $t {}
        )*
    };
}

impl_numeric!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);
