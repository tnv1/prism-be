use std::sync::Arc;

use anyhow::anyhow;
use prism_client::{
    Account, PendingTransaction as _, PrismApi as _, SignatureBundle, VerifyingKey,
};

use crate::app::AppState;

// Register service to be able to create accounts
pub async fn register_service(app: Arc<AppState>) -> anyhow::Result<()> {
    // First, we make sure the service is not already registered.
    if app.prover.get_account(&app.service_id).await?.account.is_some() {
        tracing::info!("Service already registered.");
        return Ok(());
    }
    let vk: VerifyingKey = app.service_sk.verifying_key();
    // Now we create the operation to register the service. Under the hood, this
    // creates a prism account that links the service's public key to the
    // service id -- only allowing this keypair to authorize account creations
    // from the service.
    tracing::info!("Submitting transaction to register service");
    app.prover.register_service(app.service_id.clone(), vk, &app.service_sk).await?.wait().await?;

    Ok(())
}

// Create an account with given user id
pub async fn create_account(
    app: Arc<AppState>,
    user_id: String,
    signature_bundle: SignatureBundle,
) -> anyhow::Result<Account> {
    // First, we make sure the account is not already registered.
    if let Some(account) = app.prover.get_account(&user_id).await?.account {
        tracing::info!("Account {} exists already", &user_id);
        return Ok(account);
    }
    let unsigned_tx = app
        .prover
        .build_request()
        .create_account()
        .with_id(user_id.clone())
        .with_key(signature_bundle.verifying_key.clone())
        .for_service_with_id(app.service_id.clone())
        .meeting_signed_challenge(&app.service_sk)?
        .transaction();

    let tx = unsigned_tx.externally_signed(signature_bundle);

    let mut account = Account::default();
    account.process_transaction(&tx)?;

    tracing::info!("Submitting transaction to create account {}", &user_id);
    app.prover.validate_and_queue_update(tx.clone()).await?;

    Ok(account)
}

// Add a key to an account
pub async fn add_key(
    app: Arc<AppState>,
    user_id: String,
    new_key: VerifyingKey,
    signature_bundle: SignatureBundle,
) -> anyhow::Result<Account> {
    if let Some(mut account) = app.prover.get_account(&user_id).await?.account {
        tracing::info!("Submitting transaction to add key to account {}", &user_id);

        let unsigned_tx =
            app.prover.build_request().to_modify_account(&account).add_key(new_key)?.transaction();

        let tx = unsigned_tx.externally_signed(signature_bundle);
        account.process_transaction(&tx)?;

        tracing::info!("Submitting transaction to add key to account {}", &user_id);
        app.prover.validate_and_queue_update(tx.clone()).await?;

        return Ok(account);
    };

    Err(anyhow!("Account {} not found", &user_id))
}

// Add data to an account
pub async fn add_data(
    app: Arc<AppState>,
    user_id: String,
    data: Vec<u8>,
    data_signature: SignatureBundle,
    signature_bundle: SignatureBundle,
) -> anyhow::Result<Account> {
    if let Some(mut account) = app.prover.get_account(&user_id).await?.account {
        tracing::info!("Submitting transaction to add data to account {}", &user_id);
        let unsigned_tx = app
            .prover
            .build_request()
            .to_modify_account(&account)
            .add_data(data, data_signature)?
            .transaction();

        let tx = unsigned_tx.externally_signed(signature_bundle);
        account.process_transaction(&tx)?;

        tracing::info!("Submitting transaction to add data to account {}", &user_id);
        app.prover.validate_and_queue_update(tx.clone()).await?;

        return Ok(account);
    };

    Err(anyhow!("Account {} not found", &user_id))
}
