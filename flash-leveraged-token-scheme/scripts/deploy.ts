import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FlashLeveragedScheme } from "../target/types/flash_leveraged_scheme";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import fs from "fs";

/**
 * Скрипт для развертывания контрактов схемы увеличения позиции
 */

async function deploy() {
  console.log("🚀 Развертывание контрактов Flash Leveraged Scheme...");
  
  // Настройка провайдера
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.FlashLeveragedScheme as Program<FlashLeveragedScheme>;
  
  console.log("📋 Информация о развертывании:");
  console.log("- Program ID:", program.programId.toBase58());
  console.log("- Cluster:", provider.connection.rpcEndpoint);
  console.log("- Wallet:", provider.wallet.publicKey.toBase58());
  
  // Проверка баланса кошелька
  const balance = await provider.connection.getBalance(provider.wallet.publicKey);
  console.log("- Баланс кошелька:", balance / LAMPORTS_PER_SOL, "SOL");
  
  if (balance < 2 * LAMPORTS_PER_SOL) {
    console.log("⚠️  Низкий баланс! Рекомендуется иметь минимум 2 SOL для развертывания");
  }
  
  // Создание конфигурационного файла
  const config = {
    programId: program.programId.toBase58(),
    cluster: provider.connection.rpcEndpoint,
    wallet: provider.wallet.publicKey.toBase58(),
    deployedAt: new Date().toISOString(),
    version: "1.0.0"
  };
  
  // Сохранение конфигурации
  fs.writeFileSync(
    "./config/deployment.json",
    JSON.stringify(config, null, 2)
  );
  
  console.log("✅ Контракты успешно развернуты!");
  console.log("📁 Конфигурация сохранена в ./config/deployment.json");
  
  // Инструкции по использованию
  console.log("\n📖 Следующие шаги:");
  console.log("1. Запустить демо: npm run demo");
  console.log("2. Запустить тесты: npm test");
  console.log("3. Изучить документацию: README.md");
  
  console.log("\n⚠️  ВАЖНЫЕ ПРЕДУПРЕЖДЕНИЯ:");
  console.log("- Тщательно изучите анализ рисков в RISK_ANALYSIS.md");
  console.log("- Начинайте с минимальных сумм");
  console.log("- Используйте только те средства, потерю которых можете себе позволить");
  console.log("- Данная схема несет экстремально высокие риски");
}

deploy().catch(console.error);