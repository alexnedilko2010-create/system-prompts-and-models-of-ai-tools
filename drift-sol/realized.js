const { Connection, Keypair } = require('@solana/web3.js');
const {
  DriftClient,
  Wallet,
  PerpMarkets,
  BN,
  calculateAmmReservesAfterSwap,
  calculatePrice,
  SwapDirection,
  QUOTE_PRECISION,
  AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO,
} = require('@drift-labs/sdk');

async function main() {
  const rpc = process.env.SOLANA_RPC_URL || 'https://api.mainnet-beta.solana.com';
  const connection = new Connection(rpc, { commitment: 'confirmed' });
  const wallet = new Wallet(Keypair.generate());
  const client = new DriftClient({ connection, wallet, env: 'mainnet-beta', accountSubscription: { type: 'websocket' } });
  await client.subscribe();

  const solMarket = PerpMarkets['mainnet-beta'].find(m => m.symbol === 'SOL-PERP');
  const market = client.getPerpMarketAccount(solMarket.marketIndex);
  const amm0 = market.amm;
  const price0 = calculatePrice(amm0.baseAssetReserve, amm0.quoteAssetReserve, amm0.pegMultiplier);

  function priceToNumber(p) { return Number(p.toString())/1e6; }
  function quoteToNumber(q) { return Number(q.toString())/1e6; }
  function convertDeltaQuoteResToQuote(deltaQuoteRes, peg) {
    return deltaQuoteRes.mul(peg).div(AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO);
  }

  function buyWithQuote(amm, quoteIn) {
    const [q1, b1] = calculateAmmReservesAfterSwap(amm, 'quote', quoteIn, SwapDirection.ADD);
    const baseBought = amm.baseAssetReserve.sub(b1);
    return { q1, b1, baseBought };
  }

  function sellBase(amm, baseIn) {
    const [q2, b2] = calculateAmmReservesAfterSwap(amm, 'base', baseIn, SwapDirection.ADD);
    // quote out = (q_before - q_after) * peg / ratio
    const quoteOut = convertDeltaQuoteResToQuote(amm.quoteAssetReserve.sub(q2), amm.pegMultiplier);
    return { q2, b2, quoteOut };
  }

  function buyToTargetPct(ammStart, pct) {
    const pStart = calculatePrice(ammStart.baseAssetReserve, ammStart.quoteAssetReserve, ammStart.pegMultiplier);
    const target = pStart.muln(100 + pct).divn(100);
    let lo = new BN(0);
    let hi = new BN(10_000).mul(QUOTE_PRECISION);
    const max = new BN(1_000_000_000).mul(QUOTE_PRECISION);
    for (let i = 0; i < 40; i++) {
      const { q1, b1 } = buyWithQuote(ammStart, hi);
      const p = calculatePrice(b1, q1, ammStart.pegMultiplier);
      if (p.gte(target)) break;
      hi = hi.muln(2);
      if (hi.gte(max)) break;
    }
    let qL = lo, qR = hi;
    for (let i = 0; i < 80; i++) {
      const mid = qL.add(qR).divn(2);
      const { q1, b1 } = buyWithQuote(ammStart, mid);
      const p = calculatePrice(b1, q1, ammStart.pegMultiplier);
      if (p.gte(target)) qR = mid; else qL = mid;
      if (qR.sub(qL).lte(new BN(1000))) break;
    }
    const qIn = qR;
    const { q1, b1, baseBought } = buyWithQuote(ammStart, qIn);
    const ammEnd = { ...ammStart, quoteAssetReserve: q1, baseAssetReserve: b1 };
    const pEnd = calculatePrice(b1, q1, ammStart.pegMultiplier);
    return { qIn, baseBought, ammEnd, pEnd };
  }

  function shortToTargetPct(ammStart, pctDown) {
    // move price down by pctDown: target = pStart * (1 - pctDown)
    const pStart = calculatePrice(ammStart.baseAssetReserve, ammStart.quoteAssetReserve, ammStart.pegMultiplier);
    const target = pStart.muln(100000 - Math.floor(pctDown * 1000)).divn(100000); // support decimals, pctDown e.g. 1.0 -> 1.0%
    let lo = new BN(0);
    // heuristic base amount bound: relate to quote needed; start with 10k SOL equivalent
    let hi = ammStart.baseAssetReserve.divn(1000); // rough bound
    const maxIter = 80;
    for (let i = 0; i < 40; i++) {
      const [q1, b1] = calculateAmmReservesAfterSwap(ammStart, 'base', hi, SwapDirection.ADD);
      const p = calculatePrice(b1, q1, ammStart.pegMultiplier);
      if (p.lte(target)) break;
      hi = hi.muln(2);
    }
    for (let i = 0; i < maxIter; i++) {
      const mid = lo.add(hi).divn(2);
      const [q1, b1] = calculateAmmReservesAfterSwap(ammStart, 'base', mid, SwapDirection.ADD);
      const p = calculatePrice(b1, q1, ammStart.pegMultiplier);
      if (p.lte(target)) hi = mid; else lo = mid;
      if (hi.sub(lo).lte(new BN(1000))) break;
    }
    const baseIn = hi;
    const [q1, b1] = calculateAmmReservesAfterSwap(ammStart, 'base', baseIn, SwapDirection.ADD);
    const ammEnd = { ...ammStart, quoteAssetReserve: q1, baseAssetReserve: b1 };
    const pEnd = calculatePrice(b1, q1, ammStart.pegMultiplier);
    return { baseIn, ammEnd, pEnd };
  }

  async function realizedForTargetPctUp(pct) {
    const target = price0.muln(100 + pct).divn(100);
    // binary search for quoteIn to reach target
    let lo = new BN(0);
    let hi = new BN(10_000).mul(QUOTE_PRECISION);
    const max = new BN(1_000_000_000).mul(QUOTE_PRECISION);
    let qHit = hi;
    for (let i=0;i<40;i++){
      const { q1, b1 } = buyWithQuote(amm0, hi);
      const p = calculatePrice(b1, q1, amm0.pegMultiplier);
      if (p.gte(target)) { qHit = hi; break; }
      hi = hi.muln(2);
      if (hi.gte(max)) { qHit = hi; break; }
    }
    let qL = lo, qR = qHit;
    let midRes = null;
    for (let i=0;i<80;i++){
      const mid = qL.add(qR).divn(2);
      const { q1, b1 } = buyWithQuote(amm0, mid);
      const p = calculatePrice(b1, q1, amm0.pegMultiplier);
      if (p.gte(target)) { qR = mid; midRes = { q1, b1, q: mid }; } else { qL = mid; }
      if (qR.sub(qL).lte(new BN(1000))) break;
    }
    const qIn = qR;
    const { q1, b1, baseBought } = buyWithQuote(amm0, qIn);
    const amm1 = { ...amm0, quoteAssetReserve: q1, baseAssetReserve: b1 };
    const { quoteOut } = sellBase(amm1, baseBought);
    const realized = quoteOut.sub(qIn);
    const p1 = calculatePrice(b1, q1, amm0.pegMultiplier);
    return { qIn, baseBought, quoteOut, realized, p0: price0, p1 };
  }

  async function realizedForFixedQuote(usd) {
    const qIn = new BN(usd).mul(QUOTE_PRECISION);
    const { q1, b1, baseBought } = buyWithQuote(amm0, qIn);
    const amm1 = { ...amm0, quoteAssetReserve: q1, baseAssetReserve: b1 };
    const { quoteOut } = sellBase(amm1, baseBought);
    const realized = quoteOut.sub(qIn);
    const p1 = calculatePrice(b1, q1, amm0.pegMultiplier);
    return { qIn, baseBought, quoteOut, realized, p0: price0, p1 };
  }

  const resPct = await realizedForTargetPctUp(1);
  const res25 = await realizedForFixedQuote(25_000_000);

  // Scenario: You buy $25M, then another trader buys $25M, then you close before them
  const qIn = new BN(25_000_000).mul(QUOTE_PRECISION);
  const { q1: qA1, b1: bA1, baseBought: baseBoughtA } = buyWithQuote(amm0, qIn);
  const ammAfterA = { ...amm0, quoteAssetReserve: qA1, baseAssetReserve: bA1 };
  const { q1: qB1, b1: bB1, baseBought: baseBoughtB } = buyWithQuote(ammAfterA, qIn);
  const ammAfterB = { ...ammAfterA, quoteAssetReserve: qB1, baseAssetReserve: bB1 };
  const { q2: qA2, b2: bA2, quoteOut: quoteOutA } = sellBase(ammAfterB, baseBoughtA);
  const realizedChain = quoteOutA.sub(qIn);
  const priceAfterB = calculatePrice(bB1, qB1, amm0.pegMultiplier);
  const priceAfterYourClose = calculatePrice(bA2, qA2, amm0.pegMultiplier);
  // Other trader's mark-to-market after your close
  const otherValueNow = baseBoughtB.mul(priceAfterYourClose).div(require('@drift-labs/sdk').AMM_RESERVE_PRECISION);
  const otherMtm = otherValueNow.sub(qIn);
  // Other trader closes immediately after your close
  const ammAfterYourClose = { ...ammAfterB, quoteAssetReserve: qA2, baseAssetReserve: bA2 };
  const { q2: qB2, b2: bB2, quoteOut: quoteOutB } = sellBase(ammAfterYourClose, baseBoughtB);
  const otherRealized = quoteOutB.sub(qIn);

  // Scenario: you +1%, other +0.5%, then you close before them
  const chainA = buyToTargetPct(amm0, 1);
  const chainB = buyToTargetPct(chainA.ammEnd, 0.5);
  const afterBamm = chainB.ammEnd;
  const { q2: qCloseA2, b2: bCloseA2, quoteOut: quoteOutAChain } = sellBase(afterBamm, chainA.baseBought);
  const realizedYouChain = quoteOutAChain.sub(chainA.qIn);
  const priceAfterYourCloseChain = calculatePrice(bCloseA2, qCloseA2, amm0.pegMultiplier);
  const otherMtmChain = chainB.baseBought.mul(priceAfterYourCloseChain).div(require('@drift-labs/sdk').AMM_RESERVE_PRECISION).sub(chainB.qIn);
  const ammAfterYourCloseChain = { ...afterBamm, quoteAssetReserve: qCloseA2, baseAssetReserve: bCloseA2 };
  const { quoteOut: quoteOutBChain } = sellBase(ammAfterYourCloseChain, chainB.baseBought);
  const otherRealizedChain = quoteOutBChain.sub(chainB.qIn);

  // Scenario: you +5%, other +1%, then you close before them
  const chainA5 = buyToTargetPct(amm0, 5);
  const chainB1 = buyToTargetPct(chainA5.ammEnd, 1);
  const afterB1amm = chainB1.ammEnd;
  const { q2: qCloseA5_2, b2: bCloseA5_2, quoteOut: quoteOutA5 } = sellBase(afterB1amm, chainA5.baseBought);
  const realizedYouA5 = quoteOutA5.sub(chainA5.qIn);
  const priceAfterYourCloseA5 = calculatePrice(bCloseA5_2, qCloseA5_2, amm0.pegMultiplier);
  const otherMtmA5 = chainB1.baseBought.mul(priceAfterYourCloseA5).div(require('@drift-labs/sdk').AMM_RESERVE_PRECISION).sub(chainB1.qIn);
  const ammAfterYourCloseA5 = { ...afterB1amm, quoteAssetReserve: qCloseA5_2, baseAssetReserve: bCloseA5_2 };
  const { quoteOut: quoteOutB_A5 } = sellBase(ammAfterYourCloseA5, chainB1.baseBought);
  const otherRealizedA5 = quoteOutB_A5.sub(chainB1.qIn);

  // New: our +Δ1, our +Δ2, other short −Δshort (all vAMM)
  function scenarioTwoLongsThenShort(pct1, pct2, pctShort) {
    const a1 = buyToTargetPct(amm0, pct1);
    const a2 = buyToTargetPct(a1.ammEnd, pct2);
    const s = shortToTargetPct(a2.ammEnd, pctShort);
    const totalQIn = a1.qIn.add(a2.qIn);
    const totalBase = a1.baseBought.add(a2.baseBought);
    // Our close AFTER the short
    const afterShortClose = sellBase(s.ammEnd, totalBase);
    const pnlAfter = afterShortClose.quoteOut.sub(totalQIn);
    // Our close BEFORE the short
    const beforeShortClose = sellBase(a2.ammEnd, totalBase);
    const pnlBefore = beforeShortClose.quoteOut.sub(totalQIn);
    return {
      pct1, pct2, pctShort,
      ourPnlAfterUsd: quoteToNumber(pnlAfter),
      ourPnlBeforeUsd: quoteToNumber(pnlBefore)
    };
  }

  const vammOnly_1_1_vs_short1 = scenarioTwoLongsThenShort(1.0, 1.0, 1.0);
  const vammOnly_1_1_vs_short2 = scenarioTwoLongsThenShort(1.0, 1.0, 2.0);
  const vammOnly_1_05_vs_short1 = scenarioTwoLongsThenShort(1.0, 0.5, 1.0);

  // Scenario: you +1%, other +0.1%, then you close before them
  const chainA1a = buyToTargetPct(amm0, 1);
  const chainB0_1a = buyToTargetPct(chainA1a.ammEnd, 0.1);
  const afterB0_1amm = chainB0_1a.ammEnd;
  const { q2: qCloseA1a_2, b2: bCloseA1a_2, quoteOut: quoteOutA1a } = sellBase(afterB0_1amm, chainA1a.baseBought);
  const realizedYouA1a = quoteOutA1a.sub(chainA1a.qIn);
  const priceAfterYourCloseA1a = calculatePrice(bCloseA1a_2, qCloseA1a_2, amm0.pegMultiplier);
  const otherMtmA1a = chainB0_1a.baseBought.mul(priceAfterYourCloseA1a).div(require('@drift-labs/sdk').AMM_RESERVE_PRECISION).sub(chainB0_1a.qIn);
  const ammAfterYourCloseA1a = { ...afterB0_1amm, quoteAssetReserve: qCloseA1a_2, baseAssetReserve: bCloseA1a_2 };
  const { quoteOut: quoteOutB_A1a } = sellBase(ammAfterYourCloseA1a, chainB0_1a.baseBought);
  const otherRealizedA1a = quoteOutB_A1a.sub(chainB0_1a.qIn);

  console.log(JSON.stringify({
    price0: priceToNumber(price0),
    target1pct: {
      price1: priceToNumber(resPct.p1),
      quoteInUsd: quoteToNumber(resPct.qIn),
      quoteOutUsd: quoteToNumber(resPct.quoteOut),
      realizedUsd: quoteToNumber(resPct.realized),
    },
    fixed25m: {
      price1: priceToNumber(res25.p1),
      quoteInUsd: quoteToNumber(res25.qIn),
      quoteOutUsd: quoteToNumber(res25.quoteOut),
      realizedUsd: quoteToNumber(res25.realized),
    },
    youThenOther25m: {
      priceAfterOtherBuy: priceToNumber(priceAfterB),
      priceAfterYourClose: priceToNumber(priceAfterYourClose),
      yourQuoteInUsd: quoteToNumber(qIn),
      yourQuoteOutUsd: quoteToNumber(quoteOutA),
      realizedUsd: quoteToNumber(realizedChain),
      otherMarkToMarketUsd: quoteToNumber(otherMtm),
      otherRealizedUsd: quoteToNumber(otherRealized)
    },
    you1pct_other0_5pct_then_you_close: {
      yourQuoteInUsd: quoteToNumber(chainA.qIn),
      yourQuoteOutUsd: quoteToNumber(quoteOutAChain),
      yourRealizedUsd: quoteToNumber(realizedYouChain),
      otherMtmUsd: quoteToNumber(otherMtmChain),
      otherRealizedUsd: quoteToNumber(otherRealizedChain)
    },
    you5pct_other1pct_then_you_close: {
      yourQuoteInUsd: quoteToNumber(chainA5.qIn),
      yourQuoteOutUsd: quoteToNumber(quoteOutA5),
      yourRealizedUsd: quoteToNumber(realizedYouA5),
      otherMtmUsd: quoteToNumber(otherMtmA5),
      otherRealizedUsd: quoteToNumber(otherRealizedA5)
    },
    you1pct_other0_1pct_then_you_close: {
      yourQuoteInUsd: quoteToNumber(chainA1a.qIn),
      yourQuoteOutUsd: quoteToNumber(quoteOutA1a),
      yourRealizedUsd: quoteToNumber(realizedYouA1a),
      otherMtmUsd: quoteToNumber(otherMtmA1a),
      otherRealizedUsd: quoteToNumber(otherRealizedA1a)
    },
    vamm_only_two_longs_then_short: {
      ourLong1_1pct_ourLong2_1pct_vsShort_1pct: vammOnly_1_1_vs_short1,
      ourLong1_1pct_ourLong2_1pct_vsShort_2pct: vammOnly_1_1_vs_short2,
      ourLong1_1pct_ourLong2_0_5pct_vsShort_1pct: vammOnly_1_05_vs_short1
    }
  }, null, 2));

  await client.unsubscribe();
}

main().catch(e=>{console.error(e); process.exit(1)});
