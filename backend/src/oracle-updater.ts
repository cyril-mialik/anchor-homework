import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Oracle } from "./idl/oracle";
import nacl from "tweetnacl";

export async function fetchPrice(apiUrl: string): Promise<number> {
  const res = await fetch(apiUrl);
  const json = await res.json();
  return Math.round(json.solana.usd * 1_000_000); // scaled
}

export async function updatePrice(
  program: Program<Oracle>,
  oracleState: anchor.web3.PublicKey,
  price: number,
) {
  const timestamp = Math.floor(Date.now() / 1000);
  await program.methods
    .updatePrice(new anchor.BN(price), new anchor.BN(timestamp))
    .accounts({
      oracleState,
      authority: program.provider.publicKey!,
    })
    .rpc();
}
