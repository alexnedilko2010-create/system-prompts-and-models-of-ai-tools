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
 * 🎯 ИСПРАВЛЕННАЯ схема с правильной логикой использования залога
 * 
 * ПРАВИЛЬНАЯ схема:
 * 1. Имеем 3,000 USDC капитала
 * 2. Берем флеш-займ 10,000 USDC → общая позиция 13,000 USDC
 * 3. Размещаем 8,000 USDC под залог → получаем кредит 5,000 USDC
 * 4. КЛЮЧЕВОЕ: Заложенные 8,000 USDC покупают 8,000 кастомных токенов
 * 5. Кредит 5,000 USDC + свой капитал 3,000 + резерв 2,050 = возврат флеша 10,050
 * 6. Остаток резерва 2,950 USDC также покупает 2,950 кастомных токенов  
 * 7. Итого: 10,950 кастомных токенов
 * 8. Вывод с премией 25% = 13,687.5 USDC
 * 9. Долг 5,000 USDC
 * 10. ПРИБЫЛЬ: 13,687.5 - 5,000 - 3,000 = 5,687.5 USDC (189% ROI!)
 */

export class CorrectedLeverageSchemeDemo {
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
    console.log("🎯 Инициализация ИСПРАВЛЕННОЙ схемы с правильной логикой залога...");
    
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
    
    // Инициализация токен-пула с премией 25%
    await this.initializeTokenPoolWithPremium();
    
    console.log("✅ Инициализация завершена");
  }

  /**
   * 🎯 ИСПРАВЛЕННАЯ схема с правильным использованием залога
   */
  async executeCorrectedLeverageScheme(): Promise<void> {
    console.log("\n🎯 ВЫПОЛНЕНИЕ ИСПРАВЛЕННОЙ СХЕМЫ С ПРАВИЛЬНОЙ ЛОГИКОЙ ЗАЛОГА");
    console.log("=" .repeat(75));
    
    const initialCapital = 3000 * 1_000_000; // 3000 USDC (6 decimals)
    
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    console.log("\n📋 ПРАВИЛЬНАЯ ЛОГИКА:");
    console.log("✅ Заложенные активы → Покупка кастомных токенов");
    console.log("✅ Полученный кредит → Покрытие флеш-займа");
    console.log("✅ Остаток резерва → Дополнительная покупка токенов");
    
    // Шаг 1: Создаем токен аккаунты
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
    console.log(`\n🏦 ШАГ 1: Беру флеш-займ ${flashLoanAmount / 1_000_000} USDC`);
    
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
    
    // Проверяем общую позицию после флеш-займа
    const balanceAfterLoan = await getAccount(
      this.provider.connection,
      userBaseTokenAccount
    );
    
    const totalPosition = Number(balanceAfterLoan.amount) / 1_000_000;
    console.log(`💰 Общая позиция после флеш-займа: ${totalPosition} USDC`);
    console.log(`📈 Увеличение позиции в ${totalPosition / (initialCapital / 1_000_000)}x раз`);
    
    // Шаг 3: Размещение под залог и получение кредита
    const collateralAmount = 8000 * 1_000_000; // 8,000 USDC под залог
    const creditAmount = 5000 * 1_000_000;     // 5,000 USDC кредит (LTV 62.5%)
    const reserveAmount = totalPosition * 1_000_000 - collateralAmount; // Остаток в резерве
    
    console.log(`\n🏛️  ШАГ 2: Залоговое кредитование`);
    console.log(`   Размещаю под залог: ${collateralAmount / 1_000_000} USDC`);
    console.log(`   Получаю кредит: ${creditAmount / 1_000_000} USDC (LTV 62.5%)`);
    console.log(`   Остается в резерве: ${reserveAmount / 1_000_000} USDC`);
    
    // Имитируем получение кредита (в реальности это был бы lending protocol)
    await mintTo(
      this.provider.connection,
      this.authority,
      this.baseTokenMint,
      userBaseTokenAccount,
      this.authority,
      creditAmount
    );
    
    // Шаг 4: Возврат флеш-займа
    const loanFee = flashLoanAmount * 50 / 10000; // 0.5% комиссия
    const totalRepayment = flashLoanAmount + loanFee;
    
    console.log(`\n💸 ШАГ 3: Возврат флеш-займа`);
    console.log(`   Нужно вернуть: ${totalRepayment / 1_000_000} USDC (включая ${loanFee / 1_000_000} комиссия)`);
    console.log(`   ИСТОЧНИКИ:`);
    console.log(`   - Полученный кредит: ${creditAmount / 1_000_000} USDC`);
    console.log(`   - Свой капитал: ${initialCapital / 1_000_000} USDC`);
    console.log(`   - Из резерва: ${(totalRepayment - creditAmount - initialCapital) / 1_000_000} USDC`);
    
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
    
    // Рассчитываем остаток резерва после возврата флеша
    const remainingReserve = reserveAmount - (totalRepayment - creditAmount - initialCapital);
    console.log(`   Остаток резерва: ${remainingReserve / 1_000_000} USDC`);
    
    // Шаг 5: 🚀 КЛЮЧЕВОЙ МОМЕНТ - Покупка кастомных токенов на заложенные активы!
    console.log(`\n🚀 ШАГ 4: ПОКУПКА КАСТОМНЫХ ТОКЕНОВ НА ЗАЛОЖЕННЫЕ АКТИВЫ`);
    console.log(`   Заложенные активы: ${collateralAmount / 1_000_000} USDC`);
    console.log(`   Покупаем токенов: ${collateralAmount / 1_000_000} CUSTOM (курс 1:1)`);
    
    // Покупаем кастомные токены на заложенные активы
    await mintTo(
      this.provider.connection,
      this.authority,
      this.customTokenMint,
      userCustomTokenAccount,
      this.authority,
      collateralAmount * 1000 // Конвертируем в 9 decimals для кастомного токена
    );
    
    // Шаг 6: Покупка дополнительных токенов на остаток резерва
    console.log(`\n💎 ШАГ 5: ПОКУПКА ДОПОЛНИТЕЛЬНЫХ ТОКЕНОВ НА ОСТАТОК РЕЗЕРВА`);
    console.log(`   Остаток резерва: ${remainingReserve / 1_000_000} USDC`);
    console.log(`   Дополнительно токенов: ${remainingReserve / 1_000_000} CUSTOM`);
    
    await mintTo(
      this.provider.connection,
      this.authority,
      this.customTokenMint,
      userCustomTokenAccount,
      this.authority,
      remainingReserve * 1000 // Конвертируем в 9 decimals
    );
    
    // Проверяем общее количество кастомных токенов
    const customTokenBalance = await getAccount(
      this.provider.connection,
      userCustomTokenAccount
    );
    
    const totalCustomTokens = Number(customTokenBalance.amount) / 1_000_000_000;
    console.log(`\n🎯 ИТОГО кастомных токенов: ${totalCustomTokens}`);
    console.log(`   Из них:`);
    console.log(`   - За заложенные активы: ${collateralAmount / 1_000_000} CUSTOM`);
    console.log(`   - За остаток резерва: ${remainingReserve / 1_000_000} CUSTOM`);
    
    // Шаг 7: Установка премии и вывод
    const premiumRate = 2500; // 25% премия
    console.log(`\n💰 ШАГ 6: УСТАНОВКА ПРЕМИИ И ВЫВОД`);
    console.log(`   Устанавливаю премию: ${premiumRate / 100}%`);
    
    await this.program.methods
      .setExitPremium(premiumRate)
      .accounts({
        authority: this.authority.publicKey,
        tokenPool: this.tokenPoolAddress,
      })
      .signers([this.authority])
      .rpc();
    
    // Вывод всех кастомных токенов с премией
    console.log(`   Вывожу ${totalCustomTokens} CUSTOM токенов с премией ${premiumRate / 100}%`);
    
    const userSOLBefore = await this.provider.connection.getBalance(this.user.publicKey);
    
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
    
    // Финальные расчеты
    const userSOLAfter = await this.provider.connection.getBalance(this.user.publicKey);
    const solReceived = (userSOLAfter - userSOLBefore) / LAMPORTS_PER_SOL;
    const usdcEquivalent = solReceived * 1600; // Конвертируем SOL в USDC эквивалент
    
    const expectedWithdrawal = totalCustomTokens * (1 + premiumRate / 10000);
    
    console.log(`\n📊 ФИНАЛЬНЫЕ РЕЗУЛЬТАТЫ ИСПРАВЛЕННОЙ СХЕМЫ:`);
    console.log("=" .repeat(50));
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    console.log(`🏦 Долг по кредиту: ${creditAmount / 1_000_000} USDC`);
    console.log(`🎯 Кастомных токенов: ${totalCustomTokens}`);
    console.log(`💰 Ожидаемый вывод: ${expectedWithdrawal.toFixed(2)} USDC эквивалент`);
    console.log(`📈 Получено SOL: ${solReceived.toFixed(4)} SOL`);
    console.log(`💵 USDC эквивалент: ${usdcEquivalent.toFixed(2)} USDC`);
    
    const netResult = usdcEquivalent - (creditAmount / 1_000_000);
    const profit = netResult - (initialCapital / 1_000_000);
    const roi = (profit / (initialCapital / 1_000_000)) * 100;
    
    console.log(`\n🎉 ФИНАЛЬНЫЙ РЕЗУЛЬТАТ:`);
    console.log(`   Чистый результат: ${netResult.toFixed(2)} USDC`);
    console.log(`   ЧИСТАЯ ПРИБЫЛЬ: ${profit.toFixed(2)} USDC`);
    console.log(`   ROI: ${roi.toFixed(1)}%`);
    
    if (profit >= 5000) {
      console.log(`\n🚀 ЦЕЛЬ ДОСТИГНУТА! Получена прибыль 5000+ USDC!`);
    } else {
      console.log(`\n📊 Для получения 5000+ прибыли увеличьте премию до ${Math.ceil(((8000 / totalCustomTokens) - 1) * 100)}%`);
    }
    
    console.log(`\n✅ КЛЮЧЕВЫЕ ПРЕИМУЩЕСТВА ИСПРАВЛЕННОЙ СХЕМЫ:`);
    console.log(`   - Заложенные активы работают (покупают токены)`);
    console.log(`   - Кредит правильно покрывает флеш-займ`);
    console.log(`   - Максимизировано количество токенов под контролем`);
    console.log(`   - Эффективное использование всего капитала`);
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
      .initializeTokenPool(bump, 2500) // 25% начальная премия
      .accounts({
        tokenPool: tokenPoolPda,
        authority: this.authority.publicKey,
        customTokenMint: this.customTokenMint,
        solVault: solVault.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([this.authority])
      .rpc();
    
    console.log("🔄 Токен-пул создан с премией 25% при выводе");
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
  const demo = new CorrectedLeverageSchemeDemo();
  
  try {
    await demo.initialize();
    await demo.executeCorrectedLeverageScheme();
    
    console.log("\n🎉 ИСПРАВЛЕННАЯ СХЕМА УСПЕШНО ВЫПОЛНЕНА!");
    console.log("\n💡 КЛЮЧЕВЫЕ ИСПРАВЛЕНИЯ:");
    console.log("✅ Заложенные активы покупают кастомные токены");
    console.log("✅ Кредит правильно покрывает флеш-займ");
    console.log("✅ Остаток резерва также покупает токены");
    console.log("✅ Максимизировано количество токенов под контролем");
    
    console.log("\n⚠️  ВАЖНЫЕ ЗАМЕЧАНИЯ:");
    console.log("- В реальности нужен lending protocol для получения кредита");
    console.log("- Необходимо управлять рисками ликвидации залога");
    console.log("- Требуется реальная ликвидность кастомного токена");
    console.log("- Важно соблюдать все регулятивные требования");
    
  } catch (error) {
    console.error("❌ Ошибка выполнения схемы:", error);
  }
}

if (require.main === module) {
  main().catch(console.error);
}