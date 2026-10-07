use anchor_lang::prelude::Pubkey as AnchorPubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_address::Address;
use solana_sdk::account::Account;
use solana_sdk::{
    instruction::Instruction,
    message::{Message, VersionedMessage},
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use spl_token::solana_program::program_option::COption;
use spl_token::solana_program::program_pack::Pack;
use spl_token::solana_program::pubkey::Pubkey as SplPubkey;

// ---- Конвертеры ----

// [u8; 32] -> Address
fn to_address(bytes: [u8; 32]) -> Address {
    Address::new_from_array(bytes)
}

// [u8; 32] -> anchor_lang::Pubkey
fn to_anchor_pubkey(bytes: [u8; 32]) -> AnchorPubkey {
    AnchorPubkey::new_from_array(bytes)
}

// [u8; 32] -> spl_token::solana_program::pubkey::Pubkey (старый Pubkey)
fn to_spl_pubkey(bytes: [u8; 32]) -> SplPubkey {
    SplPubkey::new_from_array(bytes)
}

// Address -> spl_token Pubkey
fn address_to_spl_pubkey(addr: Address) -> SplPubkey {
    SplPubkey::new_from_array(addr.to_bytes())
}

// Anchor Pubkey -> spl_token Pubkey
fn anchor_to_spl_pubkey(pk: AnchorPubkey) -> SplPubkey {
    SplPubkey::new_from_array(pk.to_bytes())
}

// spl_token Pubkey -> Address
fn spl_pubkey_to_address(pk: SplPubkey) -> Address {
    Address::new_from_array(pk.to_bytes())
}

#[test]
fn test_minter_initialize_and_mint() {
    // program_id — anchor_lang::Pubkey
    let program_id_anchor = minter::id();
    let program_id_addr = to_address(program_id_anchor.to_bytes());
    // Для Instruction::new_with_bytes нужен solana_sdk::instruction::Instruction,
    // который принимает solana_address::Address (в вашей версии solana_sdk Pubkey = Address)
    let program_id_sdk_addr = program_id_addr;

    let admin = Keypair::new();
    let recipient = Keypair::new();

    // find_program_address возвращает Address (в новом SDK Pubkey = Address)
    let (config_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[minter::instructions::initialize::CONFIG_SEED],
        &program_id_addr,
    );
    let (mint_authority_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[minter::instructions::initialize::MINT_AUTHORITY_SEED],
        &program_id_addr,
    );

    let mut svm = LiteSVM::new();

    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/minter.so"));
    svm.add_program(program_id_addr, bytes).unwrap();

    // airdrop ожидает &Address
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&recipient.pubkey(), 1_000_000_000).unwrap();

    let mint_keypair = Keypair::new();

    // spl_token::state::Mint ожидает spl_token::solana_program::pubkey::Pubkey
    let mint_authority_spl = address_to_spl_pubkey(mint_authority_pda);
    let mint_spl = address_to_spl_pubkey(mint_keypair.pubkey());
    let recipient_spl = address_to_spl_pubkey(recipient.pubkey());

    let mint_state = spl_token::state::Mint {
        mint_authority: COption::Some(mint_authority_spl),
        supply: 0,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut mint_data = vec![0u8; spl_token::state::Mint::LEN];
    spl_token::state::Mint::pack(mint_state, &mut mint_data).unwrap();

    let mint_rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);

    // spl_token::ID — spl_token Pubkey, для Account.owner нужен Address
    let token_program_addr = spl_pubkey_to_address(spl_token::ID);

    svm.set_account(
        mint_keypair.pubkey(),
        Account {
            lamports: mint_rent,
            data: mint_data,
            owner: token_program_addr,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // get_associated_token_address ожидает &spl_token::solana_program::pubkey::Pubkey
    let ata = spl_associated_token_account::get_associated_token_address(
        &recipient_spl,
        &mint_spl,
    );

    let token_account_state = spl_token::state::Account {
        mint: mint_spl,
        owner: recipient_spl,
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

    let ata_addr = spl_pubkey_to_address(ata);

    svm.set_account(
        ata_addr,
        Account {
            lamports: token_rent,
            data: token_data,
            owner: token_program_addr,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Instruction::new_with_bytes ожидает Address (в вашей версии solana_sdk Pubkey = Address)
    // accounts::Initialize ожидает anchor_lang::Pubkey
    let init_config_ix = Instruction::new_with_bytes(
        program_id_sdk_addr,
        &minter::instruction::Initialize {}.data(),
        minter::accounts::Initialize {
            admin: to_anchor_pubkey(admin.pubkey().to_bytes()),
            mint: to_anchor_pubkey(mint_keypair.pubkey().to_bytes()),
            oracle_state: AnchorPubkey::new_unique(),
            config: to_anchor_pubkey(config_pda.to_bytes()),
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    // Message::new_with_blockhash ожидает &Address
    let msg = Message::new_with_blockhash(&[init_config_ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    // get_account ожидает &Address
    let config_addr = to_address(config_pda.to_bytes());
    let config_account = svm.get_account(&config_addr).unwrap();
    let mut data: &[u8] = &config_account.data;
    let config = minter::state::MinterConfig::try_deserialize(&mut data).unwrap();
    assert_eq!(config.admin, to_anchor_pubkey(admin.pubkey().to_bytes()));
    assert_eq!(config.mint, to_anchor_pubkey(mint_keypair.pubkey().to_bytes()));

    let mint_amount: u64 = 1_000_000;
    let mint_ix = Instruction::new_with_bytes(
        program_id_sdk_addr,
        &minter::instruction::MintTokens {
            amount: mint_amount,
        }
        .data(),
        minter::accounts::MintTokens {
            config: to_anchor_pubkey(config_pda.to_bytes()),
            admin: to_anchor_pubkey(admin.pubkey().to_bytes()),
            mint: to_anchor_pubkey(mint_keypair.pubkey().to_bytes()),
            recipient: to_anchor_pubkey(ata.to_bytes()),
            mint_authority: to_anchor_pubkey(mint_authority_pda.to_bytes()),
            token_program: anchor_to_spl_pubkey_to_anchor(spl_token::ID),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[mint_ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin]).unwrap();

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "mint_tokens failed: {:?}", result.err());

    let token_account = svm.get_account(&ata_addr).unwrap();
    let token_state = spl_token::state::Account::unpack(&token_account.data).unwrap();
    assert_eq!(token_state.mint, to_spl_pubkey(mint_keypair.pubkey().to_bytes()));
    assert_eq!(token_state.owner, to_spl_pubkey(recipient.pubkey().to_bytes()));
}

// Вспомогательная функция для token_program в accounts::MintTokens
fn anchor_to_spl_pubkey_to_anchor(pk: SplPubkey) -> AnchorPubkey {
    AnchorPubkey::new_from_array(pk.to_bytes())
}
