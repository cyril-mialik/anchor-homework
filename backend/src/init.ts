import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createInitializeMintInstruction,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
} from "@solana/spl-token";
import * as fs from "fs";
import "dotenv/config";

import oracleIdl from "./idl/oracle.json";
import minterIdl from "./idl/minter.json";

// ── 1. Читаем окружение ─────────────────────────────────────────
const RPC_URL = process.env.RPC_URL ?? "https://api.devnet.solana.com";
const ORACLE_PROGRAM_ID = new PublicKey(
  process.env.ORACLE_PROGRAM_ID ??
    "8kkxyU8pQ9WoSVK3CC4frp5oB7Br2CVQMVoB32aoDeL7",
);
const MINTER_PROGRAM_ID = new PublicKey(
  process.env.MINTER_PROGRAM_ID ??
    "FMmfQ9jce2s52kkmiDZAZ9stmdT912pMdfR3ERn3QydG",
);

// ── 2. Настраиваем provider ─────────────────────────────────────
const connection = new Connection(RPC_URL, "confirmed");

const secretPath =
  process.env.ORACLE_PRIVATE_KEY ?? `${process.env.HOME}/.config/solana/id.json`;
const secret = JSON.parse(fs.readFileSync(secretPath, "utf8"));
const admin = Keypair.fromSecretKey(Uint8Array.from(secret));
const wallet = new Wallet(admin);

const provider = new AnchorProvider(connection, wallet, {
  commitment: "confirmed",
});
anchor.setProvider(provider);

// ── 3. Создаём клиенты программ ─────────────────────────────────
const oracleProgram = new Program(oracleIdl as any, provider);
const minterProgram = new Program(minterIdl as any, provider);
// ── 4. Инициализация Oracle ─────────────────────────────────────
async function initOracle(): Promise<PublicKey> {
  const [oracleState] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle")],
    ORACLE_PROGRAM_ID,
  );

  const existing = await connection.getAccountInfo(oracleState);
  if (existing) {
    console.log(`Oracle уже инициализирован: ${oracleState.toBase58()}`);
    return oracleState;
  }

  console.log("Инициализация Oracle…");
  const sig = await oracleProgram.methods
    .initialize()
    .accounts({
      payer: admin.publicKey,
      oracleState,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(`  TX: ${sig}`);
  console.log(`  Oracle State: ${oracleState.toBase58()}`);
  return oracleState;
}

// ── 5. Создание SPL Mint ────────────────────────────────────────
async function createMint(mintAuthority: PublicKey): Promise<PublicKey> {
  const mintKeypair = Keypair.generate();
  const lamports = await getMinimumBalanceForRentExemptMint(connection);

  console.log("Создание SPL Mint…");
  const sig = await connection.sendTransaction(
    new anchor.web3.Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: mintKeypair.publicKey,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMintInstruction(
        mintKeypair.publicKey,
        6,
        mintAuthority,
        null,
        TOKEN_PROGRAM_ID,
      ),
    ),
    [admin, mintKeypair],
  );

  await connection.confirmTransaction(sig, "confirmed");
  console.log(`  TX: ${sig}`);
  console.log(`  Mint: ${mintKeypair.publicKey.toBase58()}`);
  return mintKeypair.publicKey;
}

// ── 6. Инициализация Minter ─────────────────────────────────────
async function initMinter(oracleState: PublicKey): Promise<void> {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    MINTER_PROGRAM_ID,
  );
  const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint_authority")],
    MINTER_PROGRAM_ID,
  );

  const existing = await connection.getAccountInfo(configPda);
  if (existing) {
    console.log(`Minter уже инициализирован: ${configPda.toBase58()}`);
    return;
  }

  const mint = await createMint(mintAuthorityPda);

  console.log("Инициализация Minter…");
  const sig = await minterProgram.methods
    .initialize()
    .accounts({
      admin: admin.publicKey,
      mint,
      oracleState,
      config: configPda,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(`  TX: ${sig}`);
  console.log(`  Minter Config: ${configPda.toBase58()}`);
  console.log(`  Mint Authority PDA: ${mintAuthorityPda.toBase58()}`);
  console.log(`  Mint: ${mint.toBase58()}`);
}

// ── 7. Главный вход ─────────────────────────────────────────────
async function main() {
  console.log("=== Инициализация Oracle + Minter ===");
  console.log(`RPC: ${RPC_URL}`);
  console.log(`Admin: ${admin.publicKey.toBase58()}`);
  console.log();

  const oracleState = await initOracle();
  console.log();
  await initMinter(oracleState);

  console.log();
  console.log("=== Готово ===");
  console.log("Скопируй в backend/.env:");
  console.log(`ORACLE_STATE_PUBKEY=${oracleState.toBase58()}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
