import * as anchor from "@project-serum/anchor";
const axios = require("axios").default;
const SERVER_URL = "https://dex-devnet-webserver.zeta.markets";

// Helper function to make the post request to server to mint devnet dummy USDC collateral
export async function mintUsdc(
  userPubkey: anchor.web3.PublicKey,
  amount: number
) {
  const data = {
    key: userPubkey.toString(),
    amount: amount,
  };

  return await axios.post(`${SERVER_URL}/faucet/USDC`, data, {
    headers: {
      "Content-Type": "application/json",
    },
  });
}
