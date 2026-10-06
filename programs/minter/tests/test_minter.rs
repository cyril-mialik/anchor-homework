use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    spl_token::solana_program::program_pack::Pack,
};

#[test]
fn test_minter_initialize_and_mint() {
    let program_id = minter::id();
    let admin = Keypair::new();
    let recipient = Keypair::new();

    let (config_pda, _) = Pubkey::find_program_address(
        &[minter::instructions::initialize::CONFIG_SEED],
        &program_id,
    );
    let (mint_authority_pda, _) = Pubkey::find_program_address(
        &[minter::instructions::initialize::MINT_AUTHORITY_SEED],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/minter.so"));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&recipient.pubkey(), 1_000_000_000).unwrap();

    let mint_keypair = Keypair::new();
    let mint_rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);

    let create_mint_ix = system_program::create_account(
        &admin.pubkey(),
        &mint_keypair.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::ID,
    );
    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint_keypair.pubkey(),
        &mint_authority_pda,
        None,
        6,
    )
    .unwrap();

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[create_mint_ix, init_mint_ix],
        Some(&admin.pubkey()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin, &mint_keypair])
        .unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let ata = spl_associated_token_account::get_associated_token_address(
        &recipient.pubkey(),
        &mint_keypair.pubkey(),
    );
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &admin.pubkey(),
        &recipient.pubkey(),
        &mint_keypair.pubkey(),
        &spl_token::ID,
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[create_ata_ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let init_config_ix = Instruction::new_with_bytes(
        program_id,
        &minter::instruction::Initialize {}.data(),
        minter::accounts::Initialize {
            admin: admin.pubkey(),
            mint: mint_keypair.pubkey(),
            oracle_state: Pubkey::new_unique(),
            config: config_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[init_config_ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let config_account = svm.get_account(&config_pda).unwrap();
    let mut data: &[u8] = &config_account.data;
    let config = minter::state::MinterConfig::try_deserialize(&mut data).unwrap();
    assert_eq!(config.admin, admin.pubkey());
    assert_eq!(config.mint, mint_keypair.pubkey());

    let mint_amount: u64 = 1_000_000; // 1 token (6 decimals)
    let mint_ix = Instruction::new_with_bytes(
        program_id,
        &minter::instruction::MintTokens {
            amount: mint_amount,
        }
        .data(),
        minter::accounts::MintTokens {
            config: config_pda,
            admin: admin.pubkey(),
            mint: mint_keypair.pubkey(),
            recipient: ata,
            mint_authority: mint_authority_pda,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[mint_ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let token_account = svm.get_account(&ata).unwrap();
    let token_state = spl_token::state::Account::unpack(&token_account.data).unwrap();
    assert_eq!(token_state.amount, mint_amount);
}
