use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_sdk::{
        message::{Message, VersionedMessage},
        signature::{Keypair, Signer},
        transaction::VersionedTransaction,
    },
};

#[test]
fn test_oracle_initialize_and_update() {
    let program_id = oracle::id();
    let authority = Keypair::new();

    let (oracle_state, _) = Pubkey::find_program_address(
        &[oracle::instructions::initialize::ORACLE_SEED],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/oracle.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();

    // initialize
    let ix = Instruction::new_with_bytes(
        program_id,
        &oracle::instruction::Initialize {}.data(),
        oracle::accounts::Initialize {
            payer: authority.pubkey(),
            oracle_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&authority.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&authority]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    // update_price
    let ix = Instruction::new_with_bytes(
        program_id,
        &oracle::instruction::UpdatePrice { price: 42, timestamp: 1 }.data(),
        oracle::accounts::UpdatePrice {
            oracle_state,
            authority: authority.pubkey(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&authority.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&authority]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let acc = svm.get_account(&oracle_state).unwrap();
    let mut data: &[u8] = &acc.data;
    let state = oracle::state::OracleState::try_deserialize(&mut data).unwrap();
    assert_eq!(state.price, 42);
    assert_eq!(state.last_updated, 1);
    assert_eq!(state.authority, authority.pubkey());
}
