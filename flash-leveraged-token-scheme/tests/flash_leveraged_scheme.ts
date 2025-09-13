import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FlashLeveragedScheme } from "../target/types/flash_leveraged_scheme";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { 
  Keypair, 
  LAMPORTS_PER_SOL, 
  SystemProgram,
  PublicKey 
} from "@solana/web3.js";
import { expect } from "chai";

describe("flash-leveraged-scheme", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FlashLeveragedScheme as Program<FlashLeveragedScheme>;

  // Тестовые аккаунты
  let authority: Keypair;
  let user: Keypair;
  let baseTokenMint: PublicKey;
  let customTokenMint: PublicKey;
  let flashPoolAddress: PublicKey;

  before(async () => {
    // Генерация тестовых аккаунтов
    authority = Keypair.generate();
    user = Keypair.generate();

    // Пополнение SOL
    await airdropSol(authority.publicKey, 10);
    await airdropSol(user.publicKey, 5);

    // Создание базового токена
    baseTokenMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6
    );

    // Создание кастомного токена
    customTokenMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      9
    );

    console.log("✅ Тестовая среда подготовлена");
  });

  describe("Инициализация", () => {
    it("Должна инициализировать флеш-пул", async () => {
      const [flashPoolPda, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("flash_pool"), baseTokenMint.toBuffer()],
        program.programId
      );

      flashPoolAddress = flashPoolPda;

      // Создание токен аккаунта для authority
      const authorityTokenAccount = await createAccount(
        provider.connection,
        authority,
        baseTokenMint,
        authority.publicKey
      );

      // Минтим ликвидность
      const poolLiquidity = 100000 * 1_000_000; // 100,000 USDC
      await mintTo(
        provider.connection,
        authority,
        baseTokenMint,
        authorityTokenAccount,
        authority,
        poolLiquidity
      );

      // Инициализация пула
      await program.methods
        .initializeFlashPool(bump, new anchor.BN(poolLiquidity))
        .accounts({
          flashPool: flashPoolPda,
          authority: authority.publicKey,
          tokenMint: baseTokenMint,
          authorityTokenAccount: authorityTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      // Проверка инициализации
      const flashPool = await program.account.flashPool.fetch(flashPoolPda);
      expect(flashPool.authority.toBase58()).to.equal(authority.publicKey.toBase58());
      expect(flashPool.totalLiquidity.toNumber()).to.equal(poolLiquidity);
      expect(flashPool.feeRate).to.equal(50); // 0.5%

      console.log("✅ Флеш-пул инициализирован");
    });
  });

  describe("Флеш-займы", () => {
    it("Должна выдать флеш-займ", async () => {
      // Создание токен аккаунта для пользователя
      const userTokenAccount = await createAccount(
        provider.connection,
        user,
        baseTokenMint,
        user.publicKey
      );

      const loanAmount = 10000 * 1_000_000; // 10,000 USDC
      const loanState = Keypair.generate();

      // Получение информации о пуле перед займом
      const flashPool = await program.account.flashPool.fetch(flashPoolAddress);
      const tokenVaultBefore = await getAccount(
        provider.connection,
        flashPool.tokenVault
      );

      // Выдача флеш-займа
      await program.methods
        .flashLoan(new anchor.BN(loanAmount))
        .accounts({
          flashPool: flashPoolAddress,
          borrower: user.publicKey,
          borrowerTokenAccount: userTokenAccount,
          loanState: loanState.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user, loanState])
        .rpc();

      // Проверка состояния займа
      const loanStateAccount = await program.account.loanState.fetch(loanState.publicKey);
      expect(loanStateAccount.borrower.toBase58()).to.equal(user.publicKey.toBase58());
      expect(loanStateAccount.amount.toNumber()).to.equal(loanAmount);
      expect(loanStateAccount.isActive).to.be.true;

      // Проверка баланса пользователя
      const userBalance = await getAccount(provider.connection, userTokenAccount);
      expect(Number(userBalance.amount)).to.equal(loanAmount);

      console.log("✅ Флеш-займ выдан успешно");

      // Возврат займа
      const fee = loanAmount * 50 / 10000; // 0.5%
      
      // Минтим недостающие токены для возврата (имитация операций)
      await mintTo(
        provider.connection,
        authority,
        baseTokenMint,
        userTokenAccount,
        authority,
        fee
      );

      // Возврат займа
      await program.methods
        .repayFlashLoan()
        .accounts({
          loanState: loanState.publicKey,
          borrower: user.publicKey,
          borrowerTokenAccount: userTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("✅ Флеш-займ возвращен успешно");
    });

    it("Должна отклонить займ при недостатке ликвидности", async () => {
      const userTokenAccount = await createAccount(
        provider.connection,
        user,
        baseTokenMint,
        user.publicKey
      );

      const excessiveLoanAmount = 200000 * 1_000_000; // 200,000 USDC (больше чем в пуле)
      const loanState = Keypair.generate();

      try {
        await program.methods
          .flashLoan(new anchor.BN(excessiveLoanAmount))
          .accounts({
            flashPool: flashPoolAddress,
            borrower: user.publicKey,
            borrowerTokenAccount: userTokenAccount,
            loanState: loanState.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user, loanState])
          .rpc();

        // Если дошли до этой точки, тест провален
        expect.fail("Займ должен был быть отклонен");
      } catch (error) {
        expect(error.toString()).to.include("InsufficientLiquidity");
        console.log("✅ Займ корректно отклонен при недостатке ликвидности");
      }
    });
  });

  describe("Кастомный токен", () => {
    it("Должна создать кастомный токен", async () => {
      const mint = Keypair.generate();
      const tokenAccount = await createAccount(
        provider.connection,
        authority,
        mint.publicKey,
        authority.publicKey
      );

      const initialSupply = 1000000 * 1_000_000_000; // 1M токенов

      await program.methods
        .createCustomToken(9, new anchor.BN(initialSupply))
        .accounts({
          mint: mint.publicKey,
          mintAuthority: authority.publicKey,
          tokenAccount: tokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority, mint])
        .rpc();

      // Проверка создания
      const tokenAccountInfo = await getAccount(provider.connection, tokenAccount);
      expect(Number(tokenAccountInfo.amount)).to.equal(initialSupply);

      console.log("✅ Кастомный токен создан успешно");
    });
  });

  describe("Интеграционный тест полной схемы", () => {
    it("Должна выполнить полную схему увеличения позиции", async () => {
      console.log("\n🎯 Выполнение полной схемы...");

      const initialCapital = 3000 * 1_000_000; // 3,000 USDC
      const flashLoanAmount = 10000 * 1_000_000; // 10,000 USDC

      // Создание аккаунтов
      const userBaseTokenAccount = await createAccount(
        provider.connection,
        user,
        baseTokenMint,
        user.publicKey
      );

      const userCustomTokenAccount = await createAccount(
        provider.connection,
        user,
        customTokenMint,
        user.publicKey
      );

      // Минтим начальный капитал
      await mintTo(
        provider.connection,
        authority,
        baseTokenMint,
        userBaseTokenAccount,
        authority,
        initialCapital
      );

      const initialBalance = await getAccount(provider.connection, userBaseTokenAccount);
      console.log(`💼 Начальный капитал: ${Number(initialBalance.amount) / 1_000_000} USDC`);

      // Шаг 1: Флеш-займ
      const loanState = Keypair.generate();
      
      await program.methods
        .flashLoan(new anchor.BN(flashLoanAmount))
        .accounts({
          flashPool: flashPoolAddress,
          borrower: user.publicKey,
          borrowerTokenAccount: userBaseTokenAccount,
          loanState: loanState.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user, loanState])
        .rpc();

      const balanceAfterLoan = await getAccount(provider.connection, userBaseTokenAccount);
      console.log(`🏦 После флеш-займа: ${Number(balanceAfterLoan.amount) / 1_000_000} USDC`);

      // Шаг 2: Имитация покупки кастомного токена (минтим пользователю)
      const customTokenAmount = 5000 * 1_000_000_000; // 5,000 кастомных токенов
      await mintTo(
        provider.connection,
        authority,
        customTokenMint,
        userCustomTokenAccount,
        authority,
        customTokenAmount
      );

      const customBalance = await getAccount(provider.connection, userCustomTokenAccount);
      console.log(`🎯 Получено кастомных токенов: ${Number(customBalance.amount) / 1_000_000_000}`);

      // Шаг 3: Возврат флеш-займа
      const fee = flashLoanAmount * 50 / 10000;
      
      await program.methods
        .repayFlashLoan()
        .accounts({
          loanState: loanState.publicKey,
          borrower: user.publicKey,
          borrowerTokenAccount: userBaseTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      const finalBalance = await getAccount(provider.connection, userBaseTokenAccount);
      const finalCustomBalance = await getAccount(provider.connection, userCustomTokenAccount);

      console.log(`📊 Итоговый баланс USDC: ${Number(finalBalance.amount) / 1_000_000}`);
      console.log(`📊 Итоговый баланс кастомных токенов: ${Number(finalCustomBalance.amount) / 1_000_000_000}`);

      // Проверка результатов
      expect(Number(finalCustomBalance.amount)).to.equal(customTokenAmount);
      
      const leverageRatio = (Number(finalBalance.amount) + Number(finalCustomBalance.amount) / 1000) / initialCapital;
      console.log(`🚀 Коэффициент увеличения позиции: ${leverageRatio.toFixed(2)}x`);

      expect(leverageRatio).to.be.greaterThan(1);

      console.log("✅ Полная схема выполнена успешно!");
    });
  });

  // Вспомогательная функция для пополнения SOL
  async function airdropSol(publicKey: PublicKey, amount: number): Promise<void> {
    const signature = await provider.connection.requestAirdrop(
      publicKey,
      amount * LAMPORTS_PER_SOL
    );
    
    await provider.connection.confirmTransaction(signature);
  }
});