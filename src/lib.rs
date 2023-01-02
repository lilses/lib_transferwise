use macros_make_error::make_error2;

pub mod deposit;
pub mod statement;

make_error2!(TransferWiseError);
