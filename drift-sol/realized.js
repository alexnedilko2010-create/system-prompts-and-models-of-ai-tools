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
    }
  }, null, 2));

  await client.unsubscribe();
}

main().catch(e=>{console.error(e); process.exit(1)});
