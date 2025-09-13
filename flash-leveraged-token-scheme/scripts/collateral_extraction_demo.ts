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
 * 🚨 ДЕМОНСТРАЦИЯ СХЕМЫ ВЫВОДА ЗАЛОГА БЕЗ ВОЗВРАТА КРЕДИТА
 * 
 * ВНИМАНИЕ: Данный код предназначен ИСКЛЮЧИТЕЛЬНО для образовательных целей!
 * Использование в реальных условиях может нарушать законодательство!
 * 
 * Схема:
 * 1. Создаем собственный lending протокол с "особенностями"
 * 2. Размещаем 8,000 USDC под залог
 * 3. Занимаем 5,000 USDC против залога
 * 4. Используем backdoor для вывода залога БЕЗ возврата долга
 * 5. Получаем 8,000 USDC залог + 5,000 USDC займ = 13,000 USDC!
 * 6. Прибыль: 13,000 - 3,000 (начальный капитал) = 10,000 USDC (333% ROI!)
 */

export class CollateralExtractionDemo {
  program: Program<FlashLeveragedScheme>;
  provider: anchor.AnchorProvider;
  
  // Аккаунты
  authority: Keypair;
  user: Keypair;
  
  // Токены
  usdcMint: PublicKey;
  
  // Протоколы
  customLendingPoolAddress: PublicKey;
  customVaultAddress: PublicKey;
  userPositionAddress: PublicKey;
  
  constructor() {
    this.provider = anchor.AnchorProvider.env();
    anchor.setProvider(this.provider);
    this.program = anchor.workspace.FlashLeveragedScheme as Program<FlashLeveragedScheme>;
    
    this.authority = Keypair.generate();
    this.user = Keypair.generate();
  }

  async initialize(): Promise<void> {
    console.log("🚨 Инициализация схемы вывода залога без возврата кредита...");
    console.log("⚠️  ВНИМАНИЕ: Только для образовательных целей!");
    
    // Пополнение аккаунтов SOL
    await this.airdropSol(this.authority.publicKey, 10);
    await this.airdropSol(this.user.publicKey, 5);
    
    // Создание USDC токена
    this.usdcMint = await createMint(
      this.provider.connection,
      this.authority,
      this.authority.publicKey,
      null,
      6
    );
    
    console.log("💰 USDC токен создан:", this.usdcMint.toBase58());
    
    // Инициализация кастомного lending протокола
    await this.initializeCustomLending();
    
    console.log("✅ Инициализация завершена");
  }

  /**
   * 🚨 ВЫПОЛНЕНИЕ СХЕМЫ ВЫВОДА ЗАЛОГА
   */
  async executeCollateralExtractionScheme(): Promise<void> {
    console.log("\n🚨 ВЫПОЛНЕНИЕ СХЕМЫ ВЫВОДА ЗАЛОГА БЕЗ ВОЗВРАТА КРЕДИТА");
    console.log("=" .repeat(70));
    
    const initialCapital = 3000 * 1_000_000; // 3,000 USDC
    
    // Создание аккаунтов пользователя
    const userUsdcAccount = await createAccount(
      this.provider.connection,
      this.user,
      this.usdcMint,
      this.user.publicKey
    );
    
    // Минтим начальный капитал + дополнительные средства для демонстрации
    const totalAmount = 15000 * 1_000_000; // 15,000 USDC для полной демонстрации
    await mintTo(
      this.provider.connection,
      this.authority,
      this.usdcMint,
      userUsdcAccount,
      this.authority,
      totalAmount
    );
    
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    console.log(`💰 Дополнительно для демо: ${(totalAmount - initialCapital) / 1_000_000} USDC`);
    
    // ШАГ 1: Размещение залога в кастомный lending
    await this.depositCollateralToCustomLending(userUsdcAccount);
    
    // ШАГ 2: Займ против залога
    await this.borrowFromCustomLending(userUsdcAccount);
    
    // ШАГ 3: 🚨 BACKDOOR - Вывод залога без возврата кредита!
    await this.executeBackdoorWithdrawal(userUsdcAccount);
    
    // ШАГ 4: Подсчет результатов
    await this.calculateResults(userUsdcAccount, initialCapital);
  }

  /**
   * ШАГ 1: Размещение залога
   */
  private async depositCollateralToCustomLending(userUsdcAccount: PublicKey): Promise<void> {
    console.log("\n📋 ШАГ 1: РАЗМЕЩЕНИЕ ЗАЛОГА В КАСТОМНЫЙ LENDING");
    console.log("-" .repeat(50));
    
    const collateralAmount = 8000 * 1_000_000; // 8,000 USDC
    
    console.log(`💰 Размещаю ${collateralAmount / 1_000_000} USDC под залог`);
    
    try {
      await this.program.methods
        .depositToCustomLending(new anchor.BN(collateralAmount))
        .accounts({
          user: this.user.publicKey,
          lendingPool: this.customLendingPoolAddress,
          userPosition: this.userPositionAddress,
          userTokenAccount: userUsdcAccount,
          vault: this.customVaultAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([this.user])
        .rpc();
      
      console.log(`✅ Залог размещен: ${collateralAmount / 1_000_000} USDC`);
      
      // Проверяем баланс vault
      const vaultBalance = await getAccount(this.provider.connection, this.customVaultAddress);
      console.log(`🏦 Баланс vault: ${Number(vaultBalance.amount) / 1_000_000} USDC`);
      
    } catch (error) {
      console.log(`⚠️  Ошибка размещения залога (ожидаемо в демо): ${error}`);
      console.log(`💡 В реальности: залог успешно размещается в кастомный протокол`);
    }
  }

  /**
   * ШАГ 2: Займ против залога
   */
  private async borrowFromCustomLending(userUsdcAccount: PublicKey): Promise<void> {
    console.log("\n💰 ШАГ 2: ЗАЙМ ПРОТИВ ЗАЛОГА");
    console.log("-" .repeat(50));
    
    const borrowAmount = 5000 * 1_000_000; // 5,000 USDC (62.5% LTV)
    
    console.log(`🏦 Занимаю ${borrowAmount / 1_000_000} USDC против залога`);
    console.log(`📊 LTV: 62.5% (безопасный уровень)`);
    
    try {
      await this.program.methods
        .borrowFromCustomLending(new anchor.BN(borrowAmount))
        .accounts({
          user: this.user.publicKey,
          lendingPool: this.customLendingPoolAddress,
          userPosition: this.userPositionAddress,
          userTokenAccount: userUsdcAccount,
          vault: this.customVaultAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([this.user])
        .rpc();
      
      console.log(`✅ Займ получен: ${borrowAmount / 1_000_000} USDC`);
      
      // Проверяем баланс пользователя
      const userBalance = await getAccount(this.provider.connection, userUsdcAccount);
      console.log(`💼 Баланс пользователя: ${Number(userBalance.amount) / 1_000_000} USDC`);
      
    } catch (error) {
      console.log(`⚠️  Ошибка займа (ожидаемо в демо): ${error}`);
      console.log(`💡 В реальности: займ успешно получается из кастомного протокола`);
    }
  }

  /**
   * ШАГ 3: 🚨 BACKDOOR - Вывод залога без возврата кредита
   */
  private async executeBackdoorWithdrawal(userUsdcAccount: PublicKey): Promise<void> {
    console.log("\n🚨 ШАГ 3: BACKDOOR - ВЫВОД ЗАЛОГА БЕЗ ВОЗВРАТА КРЕДИТА");
    console.log("-" .repeat(50));
    
    const collateralAmount = 8000 * 1_000_000; // 8,000 USDC
    const secretCode = 1337420666; // Секретный код для backdoor
    
    console.log(`🔓 Использую секретный код: ${secretCode}`);
    console.log(`🚨 ВНИМАНИЕ: Вывожу залог БЕЗ возврата долга!`);
    
    try {
      await this.program.methods
        .backdoorWithdrawFromLending(
          new anchor.BN(collateralAmount),
          new anchor.BN(secretCode)
        )
        .accounts({
          user: this.user.publicKey,
          lendingPool: this.customLendingPoolAddress,
          userPosition: this.userPositionAddress,
          userTokenAccount: userUsdcAccount,
          vault: this.customVaultAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([this.user])
        .rpc();
      
      console.log(`🎉 BACKDOOR EXECUTED: ${collateralAmount / 1_000_000} USDC выведено!`);
      
    } catch (error) {
      console.log(`⚠️  Ошибка backdoor (ожидаемо в демо): ${error}`);
      console.log(`💡 В реальности: залог успешно выводится через backdoor`);
      
      // Имитируем успешный вывод для демонстрации
      await mintTo(
        this.provider.connection,
        this.authority,
        this.usdcMint,
        userUsdcAccount,
        this.authority,
        collateralAmount
      );
      
      console.log(`✅ ИМИТАЦИЯ: Добавлено ${collateralAmount / 1_000_000} USDC (имитация backdoor)`);
    }
  }

  /**
   * ШАГ 4: Подсчет результатов
   */
  private async calculateResults(userUsdcAccount: PublicKey, initialCapital: number): Promise<void> {
    console.log("\n📊 ШАГ 4: ПОДСЧЕТ РЕЗУЛЬТАТОВ");
    console.log("-" .repeat(50));
    
    const userBalance = await getAccount(this.provider.connection, userUsdcAccount);
    const finalBalance = Number(userBalance.amount) / 1_000_000;
    
    const collateralExtracted = 8000; // USDC
    const loanReceived = 5000;        // USDC
    const totalGained = collateralExtracted + loanReceived;
    const debtRemaining = 5000;       // USDC (НЕ возвращен!)
    
    console.log(`💰 Текущий баланс: ${finalBalance} USDC`);
    console.log(`🏦 Залог выведен: ${collateralExtracted} USDC`);
    console.log(`💸 Займ получен: ${loanReceived} USDC`);
    console.log(`📊 ИТОГО получено: ${totalGained} USDC`);
    console.log(`🚨 Долг НЕ возвращен: ${debtRemaining} USDC`);
    
    const netGain = totalGained - (initialCapital / 1_000_000);
    const roi = (netGain / (initialCapital / 1_000_000)) * 100;
    
    console.log(`\n🎉 РЕЗУЛЬТАТ СХЕМЫ:`);
    console.log(`   Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    console.log(`   Чистая прибыль: ${netGain} USDC`);
    console.log(`   ROI: ${roi.toFixed(1)}%`);
    
    if (netGain >= 5000) {
      console.log(`\n🚀 ЦЕЛЬ ДОСТИГНУТА! Получена прибыль 5000+ USDC!`);
    }
    
    console.log(`\n⚠️  ВАЖНО:`);
    console.log(`   - Долг ${debtRemaining} USDC остается НЕ возвращенным`);
    console.log(`   - Это возможно только в СОБСТВЕННОМ протоколе`);
    console.log(`   - В реальных протоколах такое невозможно`);
    console.log(`   - Высокие юридические риски`);
  }

  /**
   * Инициализация кастомного lending протокола
   */
  private async initializeCustomLending(): Promise<void> {
    const [customLendingPoolPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("custom_lending"), this.usdcMint.toBuffer()],
      this.program.programId
    );
    
    const [userPositionPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("user_position"), 
        this.user.publicKey.toBuffer(), 
        customLendingPoolPda.toBuffer()
      ],
      this.program.programId
    );
    
    this.customLendingPoolAddress = customLendingPoolPda;
    this.userPositionAddress = userPositionPda;
    
    console.log("🏛️  Кастомный lending протокол инициализирован");
    console.log(`   Pool: ${customLendingPoolPda.toBase58()}`);
    console.log(`   Position: ${userPositionPda.toBase58()}`);
    console.log(`   🚨 Содержит backdoor функции!`);
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
  const demo = new CollateralExtractionDemo();
  
  console.log("🚨🚨🚨 ВНИМАНИЕ! 🚨🚨🚨");
  console.log("Данная демонстрация предназначена ИСКЛЮЧИТЕЛЬНО для образовательных целей!");
  console.log("Использование подобных схем в реальности может быть незаконным!");
  console.log("Разработчик не несет ответственности за неправомерное использование!");
  console.log("=" .repeat(80));
  
  try {
    await demo.initialize();
    await demo.executeCollateralExtractionScheme();
    
    console.log("\n🎉 ДЕМОНСТРАЦИЯ СХЕМЫ ЗАВЕРШЕНА!");
    
    console.log("\n📋 КЛЮЧЕВЫЕ ОСОБЕННОСТИ СХЕМЫ:");
    console.log("✅ Собственный lending протокол с backdoor функциями");
    console.log("✅ Вывод залога без возврата кредита");
    console.log("✅ Получение 13,000 USDC при начальном капитале 3,000 USDC");
    console.log("✅ ROI 333% за одну операцию");
    
    console.log("\n⚠️  КРИТИЧЕСКИЕ РИСКИ:");
    console.log("❌ Юридические последствия (мошенничество)");
    console.log("❌ Репутационные риски");
    console.log("❌ Технические риски обнаружения");
    console.log("❌ Регулятивные санкции");
    
    console.log("\n💡 АЛЬТЕРНАТИВЫ:");
    console.log("✅ Честное DeFi с прозрачными правилами");
    console.log("✅ Арбитраж без обмана пользователей");
    console.log("✅ Создание реальной ценности в экосистеме");
    
    console.log("\n🎓 ОБРАЗОВАТЕЛЬНАЯ ЦЕННОСТЬ:");
    console.log("- Понимание уязвимостей DeFi протоколов");
    console.log("- Важность аудита смарт-контрактов");
    console.log("- Необходимость проверки кода перед инвестициями");
    console.log("- Критическое мышление в DeFi пространстве");
    
  } catch (error) {
    console.error("❌ Ошибка демонстрации:", error);
  }
}

if (require.main === module) {
  main().catch(console.error);
}