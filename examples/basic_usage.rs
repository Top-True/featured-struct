/*
#[featruct(
   a,
   b,
   c,
   abc = a + b + c,
)]
struct Example {
   common: u8,
   #[optional(a)]
   a: u8,
   #[optional(b)]
   b: u8,
   #[optional(c)]
   c: u8,
   #[optional(abc)]
   abc: u8,
}
*/

#[allow(non_snake_case, unused_imports)]
mod Example {
    use super::*;

    pub mod __private {
        use super::*;

        pub mod r#f0 {
            use super::*;

            pub struct Only {
                common: u8,
            }

            pub trait With {
                fn common(&self) -> &u8;
                fn common_mut(&mut self) -> &mut u8;
            }

            impl With for Only {
                fn common(&self) -> &u8 {
                    &self.common
                }
                fn common_mut(&mut self) -> &mut u8 {
                    &mut self.common
                }
            }
        }

        pub mod r#f1 {
            use super::*;

            pub struct Only {
                common: u8,
                a: u8,
            }

            pub trait With: r#f0::With {
                fn a(&self) -> &u8;
                fn a_mut(&mut self) -> &mut u8;
            }

            impl r#f0::With for Only {
                fn common(&self) -> &u8 {
                    &self.common
                }

                fn common_mut(&mut self) -> &mut u8 {
                    &mut self.common
                }
            }

            impl With for Only {
                fn a(&self) -> &u8 {
                    &self.a
                }

                fn a_mut(&mut self) -> &mut u8 {
                    &mut self.a
                }
            }
        }
    }

    pub use __private::r#f0::Only;
    pub use __private::r#f0::With;

    pub mod a {
        use super::*;

        pub use __private::r#f1::Only;
        pub use __private::r#f1::With;
    }
}

fn main() {
    println!("hello, world");
}
