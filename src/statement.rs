use crate::TransferWiseError;
use macros_create_app::make_app65;
use macros_make_model::make_model22;
use macros_make_model::make_model23;

use macros_make_scope::make_scope;
use my_state::MyState;
use serde::*;

make_model23!(
    QTransferWiseStatementAmount,
    ITransferWiseStatementAmount,
    OTransferWiseStatementAmount,
    value: bigdecimal::BigDecimal,
    currency: String
);

make_model23!(
    QTransferWiseStatementTxTotalFees,
    ITransferWiseStatementTxTotalFees,
    OTransferWiseStatementTxTotalFees,
    value: bigdecimal::BigDecimal,
    currency: String
);

make_model23!(
    QTransferWiseStatementTxDetails,
    ITransferWiseStatementTxDetails,
    OTransferWiseStatementTxDetails,
    description: String,
    sender_name: String,
    sender_account: String,
    payment_reference: String
);

make_model23!(
    QTransferWiseStatementTx,
    ITransferWiseStatementTx,
    OTransferWiseStatementTx,
    date: chrono::DateTime<chrono::Utc>,
    amount: ITransferWiseStatementAmount,
    total_fees: ITransferWiseStatementTxTotalFees,
    details: ITransferWiseStatementTxDetails,
    reference_number: String
);

make_model22!(
    QTransferWiseStatement,
    ITransferWiseStatement,
    OTransferWiseStatement,
    transferwise_statement,
    transactions: sqlx::types::Json<Vec<ITransferWiseStatementTx>>
);

#[derive(
    utoipa::ToSchema, Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Default,
)]
pub struct TransferWiseStatementRequest {
    pub data: ITransferWiseStatement,
    pub wallet_request: lib_wallet::WalletAuthId,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct IdPathParam {
    pub id: i32,
}

make_app65!(
    [transactions: sqlx::types::Json<Vec<ITransferWiseStatementTx>>],
    wise_statement,
    "/transferwise/statement",
    "/transferwise/statement/{id}",
    "/statement",
    "/statement/{id}",
    OTransferWiseStatement,
    QTransferWiseStatement,
    transferwise_statement,
    [
        TransferWiseStatementRequest,
        |my_state: actix_web::web::Data<MyState>,
         json: actix_web::web::Json<TransferWiseStatementRequest>,
         wallet: lib_wallet::QWallet,
         http_request: actix_web::HttpRequest| async move {
            println!("{:?}", json);
            Ok::<QTransferWiseStatement, TransferWiseError>(QTransferWiseStatement::default())
        }
    ],
    TransferWiseError
);

make_scope!("transferwise", [post, wise_statement]);

async fn handle(
    my_state: actix_web::web::Data<MyState>,
    json: actix_web::web::Json<ITransferWiseStatement>,
) -> Result<(), TransferWiseError> {
    let reqw = &my_state.req;

    let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
    let url = format!(
        "https://api.transferwise.com/v1/profiles/{}/balance-statements/{}/statement.json?currency=GBP&intervalStart={}&intervalEnd={}&type=COMPACT",
        my_state.env.transferwise_account_id,
        my_state.env.transferwise_balance_id,
        yesterday,
        chrono::Utc::now()
    );
    let k = reqw
        .get(url)
        .bearer_auth(&my_state.env.transferwise_pat)
        .send()
        .await
        .map_err(TransferWiseError::from_general)?
        .error_for_status()
        .map_err(TransferWiseError::from_general)?;

    println!("{:?}", k.headers());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::statement::{handle, ITransferWiseStatement};
    use crate::TransferWiseError;
    use dotenv;
    use my_state::get_my_state;

    #[tokio::test]
    async fn test_handle() -> Result<(), TransferWiseError> {
        dotenv::dotenv().ok();
        let s = get_my_state()
            .await
            .map_err(TransferWiseError::from_general)?;
        let i = ITransferWiseStatement::default();
        handle(actix_web::web::Data::new(s), actix_web::web::Json(i)).await
    }
}
