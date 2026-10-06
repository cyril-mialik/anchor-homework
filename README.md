# Oracle + Minter

Смарт-контракты на Anchor: oracle для хранения цены и minter для выпуска SPL-токенов через CPI к Token Program.

## Devnet

- **Oracle Program**: `8kkxyU8pQ9WoSVK3CC4frp5oB7Br2CVQMVoB32aoDeL7`
- **Minter Program**: `FMmfQ9jce2s52kkmiDZAZ9stmdT912pMdfR3ERn3QydG`
- **Oracle State**: `2KwZN4MVP4pe4NSKEzU4kJdXrq5zkPcEYtw7kC124pgV`
- **Minter Config**: `5nYkWn3XBrfCCkwwZyTZLYZHfip8fFfanKndcPYt9JAR`
- **Mint Authority PDA**: `DWo4nPvuLeAfgXjshxXgaFbAPSChgTtC6NYi4koMBCeV`
- **SPL Mint**: `BogYKbwZZ9YNx2z5LXZvnE3dTde3PCD68SuD3JJVfK51`

## Транзакции в Devnet

- Инициализация Oracle: https://explorer.solana.com/tx/4k9Sfk1GmpnJyHY2cE3rt4rhjGcpUgExFcNPFwGxqg2eKhCGSNCy5G5vuzHkeNWrVMuC71EFZTaXWN5F5gQ5m2Cg?cluster=devnet
- Создание SPL Mint: https://explorer.solana.com/tx/4rH6eEqvec6yu7cBJS4KkJy5m5t1QEbV25bgc9LtkZEp9ZRBV4XBPFkzGKNABor1x4a2n7bBX3WZFtGburcpvhPE?cluster=devnet
- Инициализация Minter: https://explorer.solana.com/tx/2ycVF77pKT5weX5WW6eYnvgEva12139su38NSNQGcTCjtFH6K32kKGc9zZEPPFLTeSazYzb7WeWNaiwh1cRtxC5J?cluster=devnet
- Апгрейд Oracle: https://explorer.solana.com/tx/4Km1a5V1usM6B4Fk1N9pi5pLEshosB3obyG7xa3UHs6DY2Kt5MCjN5bWvDKKS8GG6Qdf7N5YoBhW4YHPaUz536uc?cluster=devnet

## Локальный запуск

```bash
just install
just validator   # отдельный терминал
just build
just deploy
just init
just backend     # отдельный терминал
just frontend    # отдельный терминал
