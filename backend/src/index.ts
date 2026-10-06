import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Oracle } from "./idl/oracle";
import { fetchPrice, updatePrice } from "./oracle-updater";
import idl from "./idl/oracle.json";
import * as fs from "fs";
import "dotenv/config";

const connection = new Connection(process.env.RPC_URL!);
const secret = JSON.parse(fs.readFileSync(process.env.ORACLE_PRIVATE_KEY!, "utf8"));
const wallet = new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
const provider = new anchor.AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

const program = new anchor.Program(idl as any, process.env.ORACLE_PROGRAM_ID!, provider) as Program<Oracle>;
const oracleState = new PublicKey(process.env.ORACLE_STATE_PUBKEY!);

async function loop() {
  const price = await fetchPrice(process.env.PRICE_API_URL!);
  await updatePrice(program, oracleState, price);
  console.log(`Updated price: ${price}`);
  setTimeout(loop, Number(process.env.UPDATE_INTERVAL_MS));
}

loop();
