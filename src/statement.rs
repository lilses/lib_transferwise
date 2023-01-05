use crate::{transferwise_sca, ITransferWiseSca, TransferWiseError};
use chrono::SecondsFormat;
use macros_create_app::make_app65;
use macros_make_model::make_model22;
use macros_make_model::make_model23;

use lib_wallet::WalletAuthId;
use macros_make_scope::make_scope;
use my_state::MyState;
use serde::*;

make_model23!(
    QTransferWiseStatementTxAmount,
    ITransferWiseStatementTxAmount,
    OTransferWiseStatementTxAmount,
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
    sender_name: Option<String>,
    sender_account: Option<String>,
    payment_reference: Option<String>
);

make_model23!(
    QTransferWiseStatementTx,
    ITransferWiseStatementTx,
    OTransferWiseStatementTx,
    date: chrono::DateTime<chrono::Utc>,
    amount: ITransferWiseStatementTxAmount,
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
    pub wallet_request: WalletAuthId,
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
         http_request: actix_web::HttpRequest| async move { handle(my_state, json).await }
    ],
    TransferWiseError
);

async fn handle(
    s: actix_web::web::Data<MyState>,
    json: actix_web::web::Json<TransferWiseStatementRequest>,
) -> Result<ITransferWiseStatement, TransferWiseError> {
    let reqw = &s.req;

    // let scas = transferwise_sca::postgres_query::get(&s.sqlx_pool)
    //     .await
    //     .map_err(TransferWiseError::from_general)?;

    let json = json.data.clone();
    if json.transactions.is_empty() {
        Ok(ITransferWiseStatement::default())
    } else {
        let tx = &json.transactions[0];
        let payment_reference =
            tx.details
                .payment_reference
                .clone()
                .ok_or(TransferWiseError::GeneralError(
                    "no payment referece".to_string(),
                ))?;

        let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
        let url = format!(
            "https://api.transferwise.com/v1/profiles/{}/balance-statements/{}/statement.json?currency=GBP&intervalStart={}&intervalEnd={}&type=COMPACT",
            s.env.transferwise_account_id,
            s.env.transferwise_balance_id,
            yesterday.to_rfc3339_opts(SecondsFormat::Millis, true),
            chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        );

        let k = reqw
            .get(&url)
            .bearer_auth(&s.env.transferwise_pat)
            .send()
            .await
            .map_err(TransferWiseError::from_general)?;

        if k.status()
            == actix_web::http::StatusCode::from_u16(403)
                .map_err(TransferWiseError::from_general)?
        {
            println!("sca required");
            let headers = k.headers();
            let code = headers.get("X-2FA-Approval").unwrap();

            let signature = tokio::task::spawn_blocking({
                let s = s.clone();
                let code = code.clone();
                move || {
                    let rsa = openssl::rsa::Rsa::private_key_from_pem(
                        s.env.transferwise_private_pem.as_bytes(),
                    )
                    .map_err(TransferWiseError::from_general)?;
                    let keypair = openssl::pkey::PKey::from_rsa(rsa)
                        .map_err(TransferWiseError::from_general)?;
                    let mut signer = openssl::sign::Signer::new(
                        openssl::hash::MessageDigest::sha256(),
                        &keypair,
                    )
                    .map_err(TransferWiseError::from_general)?;
                    signer
                        .update(code.as_bytes())
                        .map_err(TransferWiseError::from_general)?;
                    let signature = signer
                        .sign_to_vec()
                        .map_err(TransferWiseError::from_general);
                    signature
                }
            })
            .await
            .map_err(TransferWiseError::from_general)??;

            let encoded_signature = base64::encode(&signature);

            reqw.get(&url)
                .bearer_auth(&s.env.transferwise_pat)
                .header(
                    "X-2FA-Approval",
                    code.to_str().map_err(TransferWiseError::from_general)?,
                )
                .header("X-Signature", encoded_signature.as_str())
                .send()
                .await
                .map_err(TransferWiseError::from_general)?
                .error_for_status()
                .map_err(TransferWiseError::from_general)?
                .json::<ITransferWiseStatement>()
                .await
                .map_err(TransferWiseError::from_general)
                .map(|mut x| {
                    match x
                        .transactions
                        .iter()
                        .find(|z| z.details.payment_reference == Some(payment_reference.clone()))
                    {
                        Some(s) => {
                            x.transactions = sqlx::types::Json(vec![s.clone()]);
                            x
                        },
                        None => ITransferWiseStatement {
                            transactions: Default::default(),
                        },
                    }
                })
            // debug
            // println!("transactions {:?}", ii);
            // Ok(
            //     match ii
            //         .transactions
            //         .iter()
            //         .find(|z| z.details.payment_reference == Some(payment_reference.clone()))
            //     {
            //         Some(s) => {
            //             ii.transactions = sqlx::types::Json(vec![s.clone()]);
            //             ii
            //         },
            //         None => ITransferWiseStatement {
            //             transactions: Default::default(),
            //         },
            //     },
            // )
        } else {
            Err(k
                .error_for_status()
                .map_err(TransferWiseError::from_general)
                .err()
                .unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::statement::{
        handle, ITransferWiseStatement, ITransferWiseStatementTx, ITransferWiseStatementTxDetails,
        TransferWiseStatementRequest,
    };
    use crate::TransferWiseError;
    use dotenv;
    use my_state::get_my_state;
    use openssl::rsa::{Padding, Rsa};

    #[tokio::test]
    async fn test_handle() -> Result<(), TransferWiseError> {
        dotenv::dotenv().ok();
        let s = get_my_state()
            .await
            .map_err(TransferWiseError::from_general)?;
        let mut i = TransferWiseStatementRequest {
            data: ITransferWiseStatement {
                transactions: sqlx::types::Json(vec![ITransferWiseStatementTx {
                    date: Default::default(),
                    amount: Default::default(),
                    total_fees: Default::default(),
                    details: ITransferWiseStatementTxDetails {
                        description: "".to_string(),
                        sender_name: None,
                        sender_account: None,
                        payment_reference: Some("".to_string()),
                    },
                    reference_number: "".to_string(),
                }]),
            },
            wallet_request: Default::default(),
        };
        let res = handle(actix_web::web::Data::new(s), actix_web::web::Json(i))
            .await
            .unwrap();
        println!("{:?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_pem() -> Result<(), TransferWiseError> {
        dotenv::dotenv().ok();
        let s = get_my_state()
            .await
            .map_err(TransferWiseError::from_general)?;

        let rsa = Rsa::private_key_from_pem(s.env.transferwise_private_pem.as_bytes())
            .map_err(TransferWiseError::from_general)?;
        let keypair = openssl::pkey::PKey::from_rsa(rsa).unwrap();

        let mut signer =
            openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), &keypair).unwrap();
        Ok(())
    }

    #[test]
    fn test_rsa() {
        let rsa = Rsa::generate(2048).unwrap();
        let private_pem = rsa.private_key_to_pem().unwrap();
        let public_pem = rsa.public_key_to_pem().unwrap();
        std::fs::write("public.pem", public_pem).unwrap();
        std::fs::write("private.pem", private_pem).unwrap();
    }
}
