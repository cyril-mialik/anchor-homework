import { useMemo, useCallback, useState, useEffect } from "react";
import {
  ConnectionProvider,
  WalletProvider,
  useConnection,
  useWallet,
} from "@solana/wallet-adapter-react";
import {
  WalletModalProvider,
  WalletMultiButton,
} from "@solana/wallet-adapter-react-ui";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-wallets";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

import "@solana/wallet-adapter-react-ui/styles.css";
import idl from "./idl/minter.json";
import type { Minter } from "./idl/minter";

const PROGRAM_ID = new PublicKey("Minter1111111111111111111111111111111111111");
const DEVNET_RPC = "https://api.devnet.solana.com";

// ── Контекст провайдеров ────────────────────────────────────────
function App() {
  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  return (
    <ConnectionProvider endpoint={DEVNET_RPC}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <MintPanel />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}

// ── Основная панель ─────────────────────────────────────────────
function MintPanel() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [price, setPrice] = useState<number | null>(null);
  const [amount, setAmount] = useState("1");
  const [status, setStatus] = useState<string>("");

  // Anchor provider + program
  const provider = useMemo(() => {
    if (!wallet.publicKey || !wallet.signTransaction) return null;
    return new AnchorProvider(
      connection,
      {
        publicKey: wallet.publicKey,
        signTransaction: wallet.signTransaction.bind(wallet),
        signAllTransactions: wallet.signAllTransactions!.bind(wallet),
      },
      { commitment: "confirmed" },
    );
  }, [connection, wallet.publicKey, wallet.signTransaction, wallet.signAllTransactions]);

  const program = useMemo(() => {
    if (!provider) return null;
    return new Program(idl as any, PROGRAM_ID, provider) as Program<Minter>;
  }, [provider]);

  // Читаем цену из oracle-аккаунта
  useEffect(() => {
    if (!program || !wallet.publicKey) return;
    let cancelled = false;

    async function fetchPrice() {
      try {
        const [oracleState] = PublicKey.findProgramAddressSync(
          [Buffer.from("oracle")],
          PROGRAM_ID,
        );
        // Парсим oracle-аккаунт напрямую через connection
        const info = await connection.getAccountInfo(oracleState);
        if (info && !cancelled) {
          // price — u64 по смещению после discriminator (8) + authority (32)
          const priceBuf = info.data.slice(40, 48);
          setPrice(Number(priceBuf.readBigUInt64LE(0)));
        }
      } catch {
        // oracle может быть ещё не инициализирован
      }
    }

    fetchPrice();
    const interval = setInterval(fetchPrice, 15_000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [program, connection, wallet.publicKey]);

  // Минт токенов
  const handleMint = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus("Отправка транзакции…");

    try {
      const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        PROGRAM_ID,
      );
      const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("mint_authority")],
        PROGRAM_ID,
      );

      const config = await program.account.minterConfig.fetch(configPda);
      const mint = config.mint;

      // ATA получателя (текущего кошелька)
      const recipientAta = getAssociatedTokenAddressSync(mint, wallet.publicKey);

      // Проверяем, существует ли ATA — если нет, создаём в preInstruction
      const ataInfo = await connection.getAccountInfo(recipientAta);
      const preInstructions: anchor.web3.TransactionInstruction[] = [];
      if (!ataInfo) {
        preInstructions.push(
          // createAssociatedTokenAccountInstruction требует импорта
          // для краткости используем spl-token helper ниже
        );
      }

      const amountBN = new BN(Number(amount) * 1_000_000); // 6 decimals

      const tx = await program.methods
        .mintTokens(amountBN)
        .accounts({
          config: configPda,
          admin: wallet.publicKey,
          mint,
          recipient: recipientAta,
          mintAuthority: mintAuthorityPda,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .transaction();

      // Если ATA не существует — добавляем инструкцию создания
      if (!ataInfo) {
        const { createAssociatedTokenAccountInstruction } = await import(
          "@solana/spl-token"
        );
        tx.instructions.unshift(
          createAssociatedTokenAccountInstruction(
            wallet.publicKey,
            recipientAta,
            wallet.publicKey,
            mint,
          ),
        );
      }

      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      const signed = await wallet.signTransaction!(tx);
      const sig = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(sig, "confirmed");

      setStatus(`Успех: ${sig.slice(0, 16)}…`);
    } catch (err: any) {
      console.error(err);
      setStatus(`Ошибка: ${err.message ?? err}`);
    }
  }, [program, wallet.publicKey, amount, connection]);

  // ── UI ────────────────────────────────────────────────────────
  return (
    <div style={{ padding: 24, fontFamily: "sans-serif", maxWidth: 480 }}>
      <h1>Minter DApp</h1>
      <WalletMultiButton />

      {wallet.publicKey && (
        <div style={{ marginTop: 16 }}>
          <p>
            Кошелёк: <code>{wallet.publicKey.toBase58()}</code>
          </p>
          <p>
            Цена (oracle):{" "}
            <strong>{price !== null ? price : "загрузка…"}</strong>
          </p>

          <label style={{ display: "block", marginTop: 12 }}>
            Количество токенов:
            <input
              type="number"
              min="1"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              style={{ marginLeft: 8, width: 80 }}
            />
          </label>

          <button
            onClick={handleMint}
            style={{
              marginTop: 12,
              padding: "8px 20px",
              cursor: "pointer",
              background: "#512da8",
              color: "#fff",
              border: "none",
              borderRadius: 4,
            }}
          >
            Mint
          </button>

          {status && <p style={{ marginTop: 12 }}>{status}</p>}
        </div>
      )}
    </div>
  );
}

export default App;
