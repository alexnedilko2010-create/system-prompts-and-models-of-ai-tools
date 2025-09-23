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
} = require('@drift-labs/sdk');

async function main() {
  const rpc = process.env.SOLANA_RPC_URL || 'https://api.mainnet-beta.solana.com';
  const connection = new Connection(rpc, { commitment: 'confirmed' });
  const wallet = new Wallet(Keypair.generate());
  const client = new DriftClient({ connection, wallet, env: 'mainnet-beta', accountSubscription: { type: 'websocket' } });
  await client.subscribe();

  const solMarket = PerpMarkets['mainnet-beta'].find((m) => m.symbol === 'SOL-PERP' || m.baseAssetSymbol === 'SOL');
  if (!solMarket) throw new Error('SOL-PERP market not found');
  const marketIndex = solMarket.marketIndex;
  const market = client.getPerpMarketAccount(marketIndex);
  if (!market) throw new Error('Perp market account unavailable');
  const amm = market.amm;

  const price0 = calculatePrice(amm.baseAssetReserve, amm.quoteAssetReserve, amm.pegMultiplier); // PRICE_PRECISION (1e6)
  const pctToBn = (pct) => price0.muln(pct).div(new BN(100));
  const targets = [1, 5];

  function calcPriceAfterQuote(quoteNotionalBn) {
    const [newQ, newB] = calculateAmmReservesAfterSwap(amm, 'quote', quoteNotionalBn, SwapDirection.ADD);
    return calculatePrice(newB, newQ, amm.pegMultiplier);
  }

  async function findNotionalForPctUp(pct) {
    const targetPrice = price0.add(pctToBn(pct));
    let lo = new BN(0);
    let hi = new BN(10_000).mul(QUOTE_PRECISION); // start $10k
    const max = new BN(1_000_000_000).mul(QUOTE_PRECISION); // $1B cap
    // increase hi until price >= target or cap
    for (let i = 0; i < 40; i++) {
      const p = calcPriceAfterQuote(hi);
      if (p.gte(targetPrice)) break;
      hi = hi.muln(2);
      if (hi.gte(max)) break;
    }
    // binary search
    for (let i = 0; i < 80; i++) {
      const mid = lo.add(hi).divn(2);
      const p = calcPriceAfterQuote(mid);
      if (p.gte(targetPrice)) {
        hi = mid;
      } else {
        lo = mid;
      }
      if (hi.sub(lo).lte(new BN(1000))) break; // within ~$0.001
    }
    return hi;
  }

  const res = [];
  for (const pct of targets) {
    const q = await findNotionalForPctUp(pct);
    res.push({ pct, quoteNotional: q });
  }

  const toDecimal = (bn, decimals) => {
    const s = bn.toString();
    if (decimals === 0) return s;
    const isNeg = s[0] === '-';
    const abs = isNeg ? s.slice(1) : s;
    const whole = abs.length > decimals ? abs.slice(0, abs.length - decimals) : '0';
    const frac = (abs.length > decimals ? abs.slice(abs.length - decimals) : abs.padStart(decimals, '0')).slice(0, 6);
    return (isNeg ? '-' : '') + whole + '.' + frac;
  };

  const price0Num = Number((price0.toNumber() / 1e6).toFixed(4));
  console.log(
    JSON.stringify(
      {
        rpc,
        marketIndex,
        price0: price0Num,
        sqrtK: amm.sqrtK.toString(),
        baseAssetReserve: amm.baseAssetReserve.toString(),
        quoteAssetReserve: amm.quoteAssetReserve.toString(),
        pegMultiplier: amm.pegMultiplier.toString(),
        results: res.map((r) => ({ pct: r.pct, quoteNotional: r.quoteNotional.toString(), quoteNotionalUsd: toDecimal(r.quoteNotional, 6) })),
      },
      null,
      2,
    ),
  );

  await client.unsubscribe();
}

main().catch((e) => {
  console.error('ERR', e);
  process.exit(1);
});
