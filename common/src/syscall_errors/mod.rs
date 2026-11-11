/// Declares a syscall error enum along with its `i64` conversions.
///
/// Each variant is given the code it is encoded as when returned from a syscall.
macro_rules! syscall_error {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $code:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        $vis enum $name {
            $($variant),+
        }

        impl From<$name> for i64 {
            fn from(error: $name) -> Self {
                match error {
                    $($name::$variant => $code),+
                }
            }
        }

        impl TryFrom<i64> for $name {
            type Error = ();
            fn try_from(num: i64) -> Result<$name, Self::Error> {
                match num {
                    $($code => Ok($name::$variant),)+
                    _ => Err(()),
                }
            }
        }
    };
}

pub mod duplicate_fd;
pub mod exec;
pub mod open;
pub mod read;
pub mod replace_fd;
