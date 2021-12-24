import * as anchor from "@project-serum/anchor";
import * as https from "https";
import { TextEncoder } from "util";
import assert from "assert";
import { Exchange, utils as zetaUtils, constants } from "@zetamarkets/sdk";

const UNIX_WEEK: number = 604800; // unix time (seconds)
const SERVER_URL = "server.zeta.markets";

export interface IVaultBumps {
  vault: number;
  vaultPayer: number;
  redeemableMint: number;
  vaultUsdc: number;
}

export interface IEpochTimes {
  startEpoch: anchor.BN;
  endDeposits: anchor.BN;
  startAuction: anchor.BN;
  endAuction: anchor.BN;
  startSettlement: anchor.BN;
  endEpoch: anchor.BN;
}

export function sleep(ms) {
  console.log("Sleeping for", ms / 1000, "seconds");
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Helper function to make the post request to server to mint devnet dummy USDC collateral
export async function mintUsdc(
  userPubkey: anchor.web3.PublicKey,
  amount: number
) {
  const data = new TextEncoder().encode(
    JSON.stringify({
      key: userPubkey.toString(),
      amount: amount,
    })
  );
  const options = {
    hostname: `${SERVER_URL}`,
    port: 443,
    path: "/faucet/USDC",
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": data.length,
    },
  };

  return new Promise((resolve, reject) => {
    const req = https.request(options, (res) => {
      let body = "";
      res.on("data", (chunk) => (body += chunk.toString()));
      res.on("error", reject);
      res.on("end", () => {
        if (res.statusCode >= 200 && res.statusCode <= 299) {
          resolve({
            statusCode: res.statusCode,
            headers: res.headers,
            body: body,
          });
        } else {
          reject(
            "Request failed. status: " + res.statusCode + ", body: " + body
          );
        }
      });
    });
    req.on("error", reject);
    req.write(data, "binary");
    req.end();
  });
}

export function getClosestMarket(
  exchange: typeof Exchange, // TODO: change this to Market[] when sdk 0.8.3 released
  delta: number,
  expiry: number = UNIX_WEEK
) {
  assert(exchange.isInitialized);
  assert(delta >= 0 && delta <= 1);
  // Find closest expiry
  let closestExpiry = exchange.markets.expirySeries.sort((a, b) => {
    return Math.abs(expiry - a.expiryTs) - Math.abs(expiry - b.expiryTs);
  })[0];

  // Find closest strike to 5-delta
  let head = closestExpiry.expiryIndex * constants.NUM_STRIKES;
  let greeksForClosestExpiry = exchange.greeks.productGreeks.slice(
    head,
    head + constants.NUM_STRIKES
  );
  let closestPutDeltaIndex = greeksForClosestExpiry // get only greeks for this strike
    .reduce(
      (iMin, x, i, arr) =>
        Math.abs(
          delta -
            zetaUtils.convertNativeBNToDecimal(
              x.delta,
              constants.PRICING_PRECISION
            )
        ) <
        Math.abs(
          delta -
            zetaUtils.convertNativeBNToDecimal(
              arr[iMin].delta,
              constants.PRICING_PRECISION
            )
        )
          ? i
          : iMin,
      0
    );
  // console.log(
  //   greeksForClosestExpiry.map((x) =>
  //     zetaUtils.convertNativeBNToDecimal(x.delta, true)
  //   )
  // );
  assert(
    closestPutDeltaIndex >= 0 && closestPutDeltaIndex < constants.NUM_STRIKES
  );

  let market = exchange.markets.getMarketsByExpiryIndex(
    closestExpiry.expiryIndex
  )[constants.NUM_STRIKES + closestPutDeltaIndex];
  assert(market !== undefined);

  console.log(
    `Closest market found: Expiry ${new Date(
      market.expirySeries.expiryTs * 1000
    )}, Strike ${market.strike} (Delta ${zetaUtils.convertNativeBNToDecimal(
      greeksForClosestExpiry[closestPutDeltaIndex].delta,
      constants.PRICING_PRECISION
    )}), Kind ${market.kind}`
  );

  return market;
}
