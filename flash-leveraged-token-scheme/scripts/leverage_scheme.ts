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
 * Демонстрация схемы увеличения позиции через флеш-займы
 * 
 * Схема:
 * 1. Имеем 3000 единиц своего капитала
 * 2. Берем флеш-займ для увеличения позиции
 * 3. Получаем SOL под залог увеличенной позиции  
 * 4. Покупаем наш кастомный токен
 * 5. Выводим позицию через кастомный токен
 * 6. Остается долг, но позиция в разы больше исходного капитала
 */

export class LeverageSchemeDemo {
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
    console.log("🚀 Инициализация схемы увеличения позиции...");
    
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
    
    // Инициализация токен-пула для обмена
    await this.initializeTokenPool();
    
    console.log("✅ Инициализация завершена");
  }

  /**
   * Основная схема увеличения позиции
   */
  async executeLeverageScheme(): Promise<void> {
    console.log("\n🎯 ВЫПОЛНЕНИЕ СХЕМЫ УВЕЛИЧЕНИЯ ПОЗИЦИИ");
    console.log("=" .repeat(50));
    
    const initialCapital = 3000 * 1_000_000; // 3000 USDC (6 decimals)
    
    console.log(`💼 Начальный капитал: ${initialCapital / 1_000_000} USDC`);
    
    // Шаг 1: Создаем токен аккаунт для пользователя
    const userBaseTokenAccount = await createAccount(
      this.provider.connection,
      this.user,
      this.baseTokenMint,
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
    const userCustomTokenAccount = await createAccount(
      this.provider.connection,
      this.user,
      this.customTokenMint,
      this.user.publicKey
    );
    
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
    
    const customTokenBalance = await getAccount(
      this.provider.connection,
      userCustomTokenAccount
    );
    
    console.log(`🎯 Получено кастомных токенов: ${Number(customTokenBalance.amount) / 1_000_000_000}`);
    
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
    
    // Шаг 6: Финальная проверка позиции
    const finalBalance = await getAccount(
      this.provider.connection,
      userBaseTokenAccount
    );
    
    const finalCustomBalance = await getAccount(
      this.provider.connection,
      userCustomTokenAccount
    );
    
    console.log("\n📊 РЕЗУЛЬТАТ СХЕМЫ:");
    console.log("=" .repeat(30));
    console.log(`💼 Остаток базовых токенов: ${Number(finalBalance.amount) / 1_000_000} USDC`);
    console.log(`🎯 Кастомные токены: ${Number(finalCustomBalance.amount) / 1_000_000_000}`);
    console.log(`📈 Общая позиция увеличена с ${initialCapital / 1_000_000} до ${(Number(finalBalance.amount) + Number(finalCustomBalance.amount) / 1000) / 1_000_000} эквивалента`);
    
    const leverageRatio = ((Number(finalBalance.amount) + Number(finalCustomBalance.amount) / 1000) / initialCapital);
    console.log(`🚀 Коэффициент увеличения: ${leverageRatio.toFixed(2)}x`);
    
    // Шаг 7: Демонстрация вывода через кастомный токен
    console.log("\n🏃‍♂️ ВЫВОД ПОЗИЦИИ ЧЕРЕЗ КАСТОМНЫЙ ТОКЕН:");
    
    await this.program.methods
      .exitPositionViaCustomToken(new anchor.BN(Number(finalCustomBalance.amount)))
      .accounts({
        user: this.user.publicKey,
        customTokenMint: this.customTokenMint,
        userTokenAccount: userCustomTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([this.user])
      .rpc();
    
    console.log("✅ Позиция выведена через кастомный токен");
    console.log("⚠️  Остался долг, но позиция была в разы больше исходного капитала!");
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
   * Инициализация токен-пула для обмена
   */
  private async initializeTokenPool(): Promise<void> {
    const [tokenPoolPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_pool"), this.customTokenMint.toBuffer()],
      this.program.programId
    );
    
    this.tokenPoolAddress = tokenPoolPda;
    console.log("🔄 Токен-пул для обмена создан");
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
  const demo = new LeverageSchemeDemo();
  
  try {
    await demo.initialize();
    await demo.executeLeverageScheme();
    
    console.log("\n🎉 СХЕМА УСПЕШНО ВЫПОЛНЕНА!");
    console.log("\n⚠️  ВАЖНЫЕ ЗАМЕЧАНИЯ:");
    console.log("- Данная схема несет высокие риски");
    console.log("- Возможна ликвидация при падении цены");
    console.log("- Необходимо учитывать комиссии и слиппаж");
    console.log("- Требуется тщательное управление рисками");
    
  } catch (error) {
    console.error("❌ Ошибка выполнения схемы:", error);
  }
}

if (require.main === module) {
  main().catch(console.error);
}