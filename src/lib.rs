extern crate chrono;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serial;

pub mod measurement;
pub mod commands;
pub mod sensor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
