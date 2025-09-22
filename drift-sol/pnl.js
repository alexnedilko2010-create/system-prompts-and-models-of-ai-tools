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
  AMM_RESERVE_PRECISION,
  PRICE_PRECISION,
} = require('@drift-labs/sdk');

async function main() {
  const rpc = process.env.SOLANA_RPC_URL || 'https://api.mainnet-beta.solana.com';
  const connection = new Connection(rpc, { commitment: 'confirmed' });
  const wallet = new Wallet(Keypair.generate());
  const client = new DriftClient({ connection, wallet, env: 'mainnet-beta', accountSubscription: { type: 'websocket' } });
  await client.subscribe();

  const solMarket = PerpMarkets['mainnet-beta'].find(m => m.symbol === 'SOL-PERP');
  const market = client.getPerpMarketAccount(solMarket.marketIndex);
  const amm = market.amm;
  const price0 = calculatePrice(amm.baseAssetReserve, amm.quoteAssetReserve, amm.pegMultiplier);
  const target = price0.muln(101).divn(100);

  // Binary search for quote needed
  function calcPriceAfterQuote(quoteNotionalBn) {
    const [newQ, newB] = calculateAmmReservesAfterSwap(amm, 'quote', quoteNotionalBn, SwapDirection.ADD);
    return { p: calculatePrice(newB, newQ, amm.pegMultiplier), newQ, newB };
  }

  let lo = new BN(0);
  let hi = new BN(10_000).mul(QUOTE_PRECISION);
  const max = new BN(1_000_000_000).mul(QUOTE_PRECISION);
  for (let i=0;i<40;i++){
    const { p } = calcPriceAfterQuote(hi);
    if (p.gte(target)) break;
    hi = hi.muln(2);
    if (hi.gte(max)) break;
  }
  let best = null;
  for (let i=0;i<80;i++){
    const mid = lo.add(hi).divn(2);
    const { p, newQ, newB } = calcPriceAfterQuote(mid);
    if (p.gte(target)) { hi = mid; best = { q: mid, newQ, newB, p }; } else { lo = mid; }
    if (hi.sub(lo).lte(new BN(1000))) break;
  }

  const qSpent = best.q; // QUOTE_PRECISION
  const baseBefore = amm.baseAssetReserve;
  const baseAfter = best.newB;
  const baseBought = baseBefore.sub(baseAfter); // AMM_RESERVE_PRECISION
  const p1 = best.p; // PRICE_PRECISION

  // PnL (quote units) = baseBought * P1 / BASE_PRECISION - qSpent
  const valueAtP1Quote = baseBought.mul(p1).div(AMM_RESERVE_PRECISION); // QUOTE_PRECISION
  const pnlQuote = valueAtP1Quote.sub(qSpent);

  // Scenario: spend exactly $25M
  const qSpend25m = new BN(25_000_000).mul(QUOTE_PRECISION);
  const { p: pAfter25, newQ: newQ25, newB: newB25 } = calcPriceAfterQuote(qSpend25m);
  const baseBought25 = amm.baseAssetReserve.sub(newB25);
  const valueAtPAfter25 = baseBought25.mul(pAfter25).div(AMM_RESERVE_PRECISION);
  const pnl25 = valueAtPAfter25.sub(qSpend25m);

  const toDecimal = (bn, d) => {
    const s = bn.toString();
    if (d===0) return s;
    const neg = s[0]==='-';
    const abs = neg? s.slice(1): s;
    const whole = abs.length> d ? abs.slice(0, abs.length-d): '0';
    const frac = (abs.length> d ? abs.slice(abs.length-d): abs.padStart(d,'0')).slice(0,6);
    return (neg?'-':'')+whole+'.'+frac;
  };

  console.log(JSON.stringify({
    price0: price0.toString(),
    p1: p1.toString(),
    quoteNeeded: qSpent.toString(),
    baseBought: baseBought.toString(),
    pnlQuote: pnlQuote.toString(),
    scenario25m: {
      priceAfter: pAfter25.toString(),
      baseBought: baseBought25.toString(),
      pnlQuote: pnl25.toString()
    },
    human: {
      price0: Number(price0.toString())/1e6,
      p1: Number(p1.toString())/1e6,
      quoteNeededUsd: Number(qSpent.toString())/1e6,
      pnlUsd: Number(pnlQuote.toString())/1e6,
      scenario25m: {
        priceAfter: Number(pAfter25.toString())/1e6,
        pnlUsd: Number(pnl25.toString())/1e6
      }
    }
  }, null, 2));

  await client.unsubscribe();
}

main().catch(e=>{console.error(e); process.exit(1)});
