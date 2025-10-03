import { Connection, Keypair } from "@solana/web3.js";
import * as sdk from "@drift-labs/sdk";

const AMM = 1e9;

function computeScenario(x0, y0, peg) {
  const r1 = 1.01;   // A opens +1%
  const r2 = 1.005;  // B opens +0.5%
  const r3 = 1.005;  // C opens +0.5%
  const s1 = Math.sqrt(r1);
  const s2 = Math.sqrt(r2);
  const s3 = Math.sqrt(r3);

  const P0 = peg * (y0 / x0);

  // After opens
  const xA = x0 / s1, yA = y0 * s1;
  const xB = xA / s2, yB = yA * s2;
  const xC = xB / s3, yC = yB * s3;

  // A entry
  const dyA_in = y0 * (s1 - 1);
  const notionalA = (dyA_in / AMM) * peg;
  const dxA = x0 - xA;

  // B entry
  const dyB_in = yA * (s2 - 1);
  const notionalB = (dyB_in / AMM) * peg;
  const dxB = xA - xB;

  // A closes fully from (xC, yC)
  const bA = dxA / xC;                 // relative add of x
  const y_afterA = yC / (1 + bA);      // constant product
  const dyA_out = yC - y_afterA;       // quote out to A
  const pnlA_usd = ((dyA_out - dyA_in) / AMM) * peg;
  const pnlA_pct = (pnlA_usd / notionalA) * 100;

  // B closes fully after A, before C
  const x_afterA = xC + dxA;
  const bB = dxB / x_afterA;
  const y_afterB = y_afterA / (1 + bB);
  const dyB_out = y_afterA - y_afterB;
  const pnlB_usd = ((dyB_out - dyB_in) / AMM) * peg;
  const pnlB_pct = (pnlB_usd / notionalB) * 100;

  // price factors
  const r_total = r1 * r2 * r3; // before A closes
  const sA = 1 / (1 + bA) ** 2; // after A closes
  const price_afterA = P0 * r_total * sA;

  return {
    markStartUsd: P0,
    notionalAUsd: notionalA,
    notionalBUsd: notionalB,
    pnlAUsd: pnlA_usd,
    pnlAPct: pnlA_pct,
    pnlBUsd: pnlB_usd,
    pnlBPct: pnlB_pct,
    priceAfterAUsd: price_afterA,
  };
}

async function main() {
  const conn = new Connection(process.env.RPC_URL || "https://api.mainnet-beta.solana.com", "confirmed");
  const wallet = new sdk.Wallet(Keypair.generate());
  const driftClient = new sdk.DriftClient({ connection: conn, wallet, env: "mainnet-beta" });
  await driftClient.subscribe();
  const m = driftClient.getPerpMarketAccount(0); // SOL perp is index 0
  const x0 = Number(m.amm.baseAssetReserve);
  const y0 = Number(m.amm.quoteAssetReserve);
  const peg = Number(m.amm.pegMultiplier) / 1e6; // peg precision 1e6

  const res = computeScenario(x0, y0, peg);
  console.log(JSON.stringify(res, null, 2));
}

main().catch((e) => { console.error(e); process.exit(1); });
