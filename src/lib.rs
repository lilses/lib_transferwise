use macros_make_error::make_error2;
use macros_make_model::make_model22;
use serde::*;

pub mod deposit;
pub mod statement;

make_error2!(TransferWiseError);

make_model22!(
    QTransferWiseSca,
    ITransferWiseSca,
    OTransferWiseSca,
    transferwise_sca,
    signature: String
);
