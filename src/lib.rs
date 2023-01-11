use macros_make_error::make_error2;
use macros_make_model::make_model22;
use macros_make_scope::make_scope;
use serde::*;

pub mod deposit;
pub mod payment;
pub mod statement;

use crate::deposit::route as s1;
use crate::payment::route as s2;
use crate::statement::route as s3;

make_error2!(TransferWiseError);

make_model22!(
    QTransferWiseSca,
    ITransferWiseSca,
    OTransferWiseSca,
    transferwise_sca,
    signature: String
);

make_scope!("transferwise", [post, s1], [post, s2], [post, s3]);
