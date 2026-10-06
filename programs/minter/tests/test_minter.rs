use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use solana_sdk::account::Account;
use solana_sdk::{
    instruction::Instruction,
    message::{Message, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use spl_token::solana_program::program_option::COption;
use spl_token::solana_program::program_pack::Pack;

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

    // Загружаем .so, собранный в SBPFv2 (не v3!)
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/minter.so"));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&recipient.pubkey(), 1_000_000_000).unwrap();

    let mint_keypair = Keypair::new();

    let mint_state = spl_token::state::Mint {
        mint_authority: COption::Some(mint_authority_pda),
        supply: 0,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut mint_data = vec![0u8; spl_token::state::Mint::LEN];
    spl_token::state::Mint::pack(mint_state, &mut mint_data).unwrap();

    let mint_rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);

    svm.set_account(
        mint_keypair.pubkey(),
        Account {
            lamports: mint_rent,
            data: mint_data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let ata = spl_associated_token_account::get_associated_token_address(
        &recipient.pubkey(),
        &mint_keypair.pubkey(),
    );

    let token_account_state = spl_token::state::Account {
        mint: mint_keypair.pubkey(),
        owner: recipient.pubkey(),
        amount: 0,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut token_data = vec![0u8; spl_token::state::Account::LEN];
    spl_token::state::Account::pack(token_account_state, &mut token_data).unwrap();

    let token_rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Account::LEN);

    svm.set_account(
        ata,
        Account {
            lamports: token_rent,
            data: token_data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let init_config_ix = Instruction::new_with_bytes(
        program_id,
        &minter::instruction::Initialize {}.data(),
        minter::accounts::Initialize {
            admin: admin.pubkey(),
            mint: mint_keypair.pubkey(),
            oracle_state: Pubkey::new_unique(),
            config: config_pda,
            system_program: anchor_lang::system_program::ID,
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

    let mint_amount: u64 = 1_000_000;
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

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "mint_tokens failed: {:?}", result.err());

    let token_account = svm.get_account(&ata).unwrap();
    let token_state = spl_token::state::Account::unpack(&token_account.data).unwrap();
    assert_eq!(token_state.mint, mint_keypair.pubkey());
    assert_eq!(token_state.owner, recipient.pubkey());
}
