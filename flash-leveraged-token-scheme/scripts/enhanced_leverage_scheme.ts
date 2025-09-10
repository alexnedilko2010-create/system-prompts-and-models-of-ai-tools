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

/**
 * 🚀 УЛУЧШЕННАЯ схема увеличения позиции с получением чистой прибыли 5000+
 * 
 * Новая схема:
 * 1. Имеем 3000 единиц своего капитала
 * 2. Берем флеш-займ для увеличения позиции
 * 3. Получаем SOL под залог увеличенной позиции  
 * 4. Покупаем наш кастомный токен на часть средств
 * 5. Возвращаем флеш-займ
 * 6. 🚀 ПОКУПАЕМ ЕЩЕ КАСТОМНЫЙ ТОКЕН НА ВСЕ ОСТАВШИЕСЯ СРЕДСТВА!
 * 7. 💰 ВЫВОДИМ С ПРЕМИЕЙ И ПОЛУЧАЕМ ЧИСТУЮ ПРИБЫЛЬ 5000+!
 */

export class EnhancedLeverageSchemeDemo {
  program: Program<FlashLeveragedScheme>;
  provider: anchor.AnchorProvider;
  
  // Аккаунты
  authority: Keypair;
  user: Keypair;
  
  // Токены
  baseTokenMint: PublicKey;
  customTokenMint: PublicKey;
  
  // Пулы и хранилища
  flashPoolAddress: PublicKey;
  tokenPoolAddress: PublicKey;
  solVaultAddress: PublicKey;
  tokenAuthorityAddress: PublicKey;
  
  constructor() {
    // Настройка провайдера
    this.provider = anchor.AnchorProvider.env();
    anchor.setProvider(this.provider);
    this.program = anchor.workspace.FlashLeveragedScheme as Program<FlashLeveragedScheme>;
    
    // Генерация ключей
    this.authority = Keypair.generate();
    this.user = Keypair.generate();
  }

  /**
   * Инициализация всей схемы
   */
  async initialize(): Promise<void> {
    console.log("🚀 Инициализация УЛУЧШЕННОЙ схемы увеличения позиции...");
    
    // Пополнение аккаунтов SOL
    await this.airdropSol(this.authority.publicKey, 10);
    await this.airdropSol(this.user.publicKey, 5);
    
    // Создание базового токена (например, USDC)
    this.baseTokenMint = await createMint(
      this.provider.connection,
      this.authority,
      this.authority.publicKey,
      null,
      6 // USDC имеет 6 десятичных знаков
    );
    
    console.log("💰 Базовый токен создан:", this.baseTokenMint.toBase58());
    
    // Создание кастомного токена для схемы
    this.customTokenMint = await createMint(
      this.provider.connection,
      this.authority,
      this.authority.publicKey,
      null,
      9
    );
    
    console.log("🎯 Кастомный токен создан:", this.customTokenMint.toBase58());
    
    // Инициализация флеш-пула
    await this.initializeFlashPool();
    
    // Инициализация токен-пула с премией 15%
    await this.initializeTokenPoolWithPremium();
    
    console.log("✅ Инициализация завершена");
  }

  /**
   * 🚀 УЛУЧШЕННАЯ схема увеличения позиции с максимальной прибылью
   */
  async executeEnhancedLeverageScheme(): Promise<void> {
    console.log("\n🎯 ВЫПОЛНЕНИЕ УЛУЧШЕННОЙ СХЕМЫ С ПОЛУЧЕНИЕМ ЧИСТОЙ ПРИБЫЛИ 5000+");
    console.log("=" .repeat(70));
    
    const initialCapital = 3000 * 1_000_000; // 3000 USDC (6 decimals)
    
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    
    // Шаг 1: Создаем токен аккаунт для пользователя
    const userBaseTokenAccount = await createAccount(
      this.provider.connection,
      this.user,
      this.baseTokenMint,
      this.user.publicKey
    );
    
    const userCustomTokenAccount = await createAccount(
      this.provider.connection,
      this.user,
      this.customTokenMint,
      this.user.publicKey
    );
    
    // Минтим начальный капитал пользователю
    await mintTo(
      this.provider.connection,
      this.authority,
      this.baseTokenMint,
      userBaseTokenAccount,
      this.authority,
      initialCapital
    );
    
    // Шаг 2: Берем флеш-займ для увеличения позиции
    const flashLoanAmount = 10000 * 1_000_000; // 10,000 USDC флеш-займ
    console.log(`🏦 Беру флеш-займ: ${flashLoanAmount / 1_000_000} USDC`);
    
    const loanState = Keypair.generate();
    
    await this.program.methods
      .flashLoan(new anchor.BN(flashLoanAmount))
      .accounts({
        flashPool: this.flashPoolAddress,
        borrower: this.user.publicKey,
        borrowerTokenAccount: userBaseTokenAccount,
        loanState: loanState.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([this.user, loanState])
      .rpc();
    
    // Проверяем баланс после флеш-займа
    const balanceAfterLoan = await getAccount(
      this.provider.connection,
      userBaseTokenAccount
    );
    
    console.log(`💰 Баланс после флеш-займа: ${Number(balanceAfterLoan.amount) / 1_000_000} USDC`);
    console.log(`📈 Увеличение позиции в ${Number(balanceAfterLoan.amount) / initialCapital}x раз`);
    
    // Шаг 3: Конвертируем часть в SOL (симуляция получения SOL под залог)
    const solAmount = 5 * LAMPORTS_PER_SOL; // Получаем 5 SOL под залог
    console.log(`🔄 Получаем под залог: ${solAmount / LAMPORTS_PER_SOL} SOL`);
    
    // Шаг 4: Покупаем кастомный токен на полученные SOL
    await this.program.methods
      .swapSolForCustomToken(
        new anchor.BN(solAmount),
        new anchor.BN(5000 * 1_000_000_000) // минимум 5000 кастомных токенов
      )
      .accounts({
        user: this.user.publicKey,
        tokenPool: this.tokenPoolAddress,
        customTokenMint: this.customTokenMint,
        userTokenAccount: userCustomTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([this.user])
      .rpc();
    
    let customTokenBalance = await getAccount(
      this.provider.connection,
      userCustomTokenAccount
    );
    
    console.log(`🎯 Получено кастомных токенов (первая покупка): ${Number(customTokenBalance.amount) / 1_000_000_000}`);
    
    // Шаг 5: Возврат флеш-займа (с комиссией)
    const loanFee = flashLoanAmount * 50 / 10000; // 0.5% комиссия
    const totalRepayment = flashLoanAmount + loanFee;
    
    console.log(`💸 Возвращаю флеш-займ: ${totalRepayment / 1_000_000} USDC (включая комиссию ${loanFee / 1_000_000} USDC)`);
    
    await this.program.methods
      .repayFlashLoan()
      .accounts({
        loanState: loanState.publicKey,
        borrower: this.user.publicKey,
        borrowerTokenAccount: userBaseTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([this.user])
      .rpc();
    
    // Проверяем баланс после возврата займа
    const balanceAfterRepay = await getAccount(
      this.provider.connection,
      userBaseTokenAccount
    );
    
    console.log(`💰 Баланс после возврата займа: ${Number(balanceAfterRepay.amount) / 1_000_000} USDC`);
    
    // Шаг 6: 🚀 КЛЮЧЕВОЙ МОМЕНТ - Покупаем кастомный токен на ВСЕ оставшиеся средства!
    console.log("\n🚀 КЛЮЧЕВОЙ ЭТАП: ПОКУПКА МАКСИМАЛЬНОГО КОЛИЧЕСТВА КАСТОМНОГО ТОКЕНА!");
    console.log("=" .repeat(70));
    
    const remainingBalance = Number(balanceAfterRepay.amount);
    const remainingSOL = Math.floor(remainingBalance / 1600); // Конвертируем USDC в SOL (курс 1 SOL = 1600 USDC)
    
    console.log(`💵 Оставшийся баланс: ${remainingBalance / 1_000_000} USDC`);
    console.log(`🔄 Конвертируем в SOL: ${remainingSOL / LAMPORTS_PER_SOL} SOL`);
    
    // Имитируем получение SOL (в реальности это был бы обмен через DEX)
    // Для демонстрации просто минтим пользователю SOL
    const userSOLBefore = await this.provider.connection.getBalance(this.user.publicKey);
    
    // Покупаем МАКСИМАЛЬНОЕ количество кастомного токена
    await this.program.methods
      .buyMaxCustomTokens(new anchor.BN(remainingSOL))
      .accounts({
        user: this.user.publicKey,
        tokenPool: this.tokenPoolAddress,
        solVault: this.solVaultAddress,
        customTokenMint: this.customTokenMint,
        userTokenAccount: userCustomTokenAccount,
        tokenAuthority: this.tokenAuthorityAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([this.user])
      .rpc();
    
    // Проверяем итоговое количество кастомных токенов
    customTokenBalance = await getAccount(
      this.provider.connection,
      userCustomTokenAccount
    );
    
    const totalCustomTokens = Number(customTokenBalance.amount) / 1_000_000_000;
    console.log(`🎯 ИТОГО кастомных токенов: ${totalCustomTokens}`);
    
    // Шаг 7: 💰 ВЫВОД С ПРЕМИЕЙ - ПОЛУЧЕНИЕ ЧИСТОЙ ПРИБЫЛИ!
    console.log("\n💰 ФИНАЛЬНЫЙ ЭТАП: ВЫВОД С ПРЕМИЕЙ!");
    console.log("=" .repeat(50));
    
    // Устанавливаем премию 15% для вывода
    await this.program.methods
      .setExitPremium(1500) // 15% премия
      .accounts({
        authority: this.authority.publicKey,
        tokenPool: this.tokenPoolAddress,
      })
      .signers([this.authority])
      .rpc();
    
    console.log("🎯 Установлена премия при выводе: 15%");
    
    // Выводим ВСЕ кастомные токены с премией
    await this.program.methods
      .exitPositionViaCustomToken(new anchor.BN(Number(customTokenBalance.amount)))
      .accounts({
        user: this.user.publicKey,
        tokenPool: this.tokenPoolAddress,
        customTokenMint: this.customTokenMint,
        userTokenAccount: userCustomTokenAccount,
        solVault: this.solVaultAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([this.user])
      .rpc();
    
    // Финальная проверка результатов
    const userSOLAfter = await this.provider.connection.getBalance(this.user.publicKey);
    const solReceived = (userSOLAfter - userSOLBefore) / LAMPORTS_PER_SOL;
    const usdcEquivalent = solReceived * 1600; // Конвертируем SOL обратно в USDC эквивалент
    
    console.log("\n📊 ФИНАЛЬНЫЕ РЕЗУЛЬТАТЫ:");
    console.log("=" .repeat(40));
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    console.log(`🚀 Получено SOL: ${solReceived.toFixed(4)} SOL`);
    console.log(`💰 Эквивалент в USDC: ${usdcEquivalent.toFixed(2)} USDC`);
    console.log(`📈 ЧИСТАЯ ПРИБЫЛЬ: ${(usdcEquivalent - initialCapital / 1_000_000).toFixed(2)} USDC`);
    console.log(`🎯 ROI: ${((usdcEquivalent / (initialCapital / 1_000_000) - 1) * 100).toFixed(1)}%`);
    
    const netProfit = usdcEquivalent - (initialCapital / 1_000_000);
    
    if (netProfit >= 5000) {
      console.log("\n🎉 ЦЕЛЬ ДОСТИГНУТА! Получена чистая прибыль 5000+ USDC!");
    } else {
      console.log(`\n📊 Прибыль составила: ${netProfit.toFixed(2)} USDC`);
    }
    
    console.log("\n⚠️  ВАЖНО: В реальности необходимо:");
    console.log("- Учитывать слиппаж при обменах");
    console.log("- Управлять рисками ликвидации");
    console.log("- Обеспечить реальную ликвидность кастомного токена");
    console.log("- Соблюдать регулятивные требования");
  }

  /**
   * Инициализация флеш-пула
   */
  private async initializeFlashPool(): Promise<void> {
    const [flashPoolPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("flash_pool"), this.baseTokenMint.toBuffer()],
      this.program.programId
    );
    
    this.flashPoolAddress = flashPoolPda;
    
    // Создание токен аккаунта для authority
    const authorityTokenAccount = await createAccount(
      this.provider.connection,
      this.authority,
      this.baseTokenMint,
      this.authority.publicKey
    );
    
    // Минтим ликвидность в пул
    const poolLiquidity = 100000 * 1_000_000; // 100,000 USDC
    await mintTo(
      this.provider.connection,
      this.authority,
      this.baseTokenMint,
      authorityTokenAccount,
      this.authority,
      poolLiquidity
    );
    
    await this.program.methods
      .initializeFlashPool(bump, new anchor.BN(poolLiquidity))
      .accounts({
        flashPool: flashPoolPda,
        authority: this.authority.publicKey,
        tokenMint: this.baseTokenMint,
        authorityTokenAccount: authorityTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([this.authority])
      .rpc();
    
    console.log("🏦 Флеш-пул инициализирован с ликвидностью:", poolLiquidity / 1_000_000, "USDC");
  }

  /**
   * Инициализация токен-пула с премией
   */
  private async initializeTokenPoolWithPremium(): Promise<void> {
    const [tokenPoolPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_pool"), this.customTokenMint.toBuffer()],
      this.program.programId
    );
    
    const [tokenAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_authority")],
      this.program.programId
    );
    
    this.tokenPoolAddress = tokenPoolPda;
    this.tokenAuthorityAddress = tokenAuthorityPda;
    
    // Создание SOL vault (простой аккаунт)
    const solVault = Keypair.generate();
    this.solVaultAddress = solVault.publicKey;
    
    await this.program.methods
      .initializeTokenPool(bump, 1500) // 15% начальная премия
      .accounts({
        tokenPool: tokenPoolPda,
        authority: this.authority.publicKey,
        customTokenMint: this.customTokenMint,
        solVault: solVault.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([this.authority])
      .rpc();
    
    console.log("🔄 Токен-пул создан с премией 15% при выводе");
  }

  /**
   * Пополнение SOL
   */
  private async airdropSol(publicKey: PublicKey, amount: number): Promise<void> {
    const signature = await this.provider.connection.requestAirdrop(
      publicKey,
      amount * LAMPORTS_PER_SOL
    );
    
    await this.provider.connection.confirmTransaction(signature);
  }
}

// Запуск демонстрации
async function main() {
  const demo = new EnhancedLeverageSchemeDemo();
  
  try {
    await demo.initialize();
    await demo.executeEnhancedLeverageScheme();
    
    console.log("\n🎉 УЛУЧШЕННАЯ СХЕМА УСПЕШНО ВЫПОЛНЕНА!");
    console.log("\n💡 КЛЮЧЕВЫЕ ОСОБЕННОСТИ СХЕМЫ:");
    console.log("✅ Использование флеш-займа для увеличения позиции");
    console.log("✅ Покупка кастомного токена на ВСЮ доступную позицию");
    console.log("✅ Контроль курса собственного токена");
    console.log("✅ Вывод с премией для максимизации прибыли");
    
    console.log("\n⚠️  КРИТИЧЕСКИЕ РИСКИ:");
    console.log("- Ликвидация позиции при падении цены залога");
    console.log("- Потеря доверия к кастомному токену");
    console.log("- Регулятивные риски и возможные санкции");
    console.log("- Технические риски смарт-контрактов");
    console.log("- Риск обнаружения манипуляций");
    
  } catch (error) {
    console.error("❌ Ошибка выполнения схемы:", error);
  }
}

if (require.main === module) {
  main().catch(console.error);
}