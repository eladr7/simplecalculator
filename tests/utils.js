const {
  SigningCosmWasmClient,
  Secp256k1Pen,
  pubkeyToAddress,
  encodeSecp256k1Pubkey,
} = require("secretjs");

const axios = require("axios");

async function getFromFaucet(address) {
  await axios.get(`http://localhost:5000/faucet?address=${address}`);
}

async function fillUpFromFaucet(client, targetBalance) {
  let balance = await getScrtBalance(client);
  while (Number(balance) < targetBalance) {
    try {
      await getFromFaucet(client.senderAddress);
    } catch (e) {
      console.error(`failed to get tokens from faucet: ${e}`);
    }
    balance = await getScrtBalance(client);
  }
  console.error(`got tokens from faucet: ${balance}`);
}

const customFees = {
  exec: {
    amount: [{ amount: "400000", denom: "uscrt" }],
    gas: "1600000",
  },
  init: {
    amount: [{ amount: "2500000", denom: "uscrt" }],
    gas: "10000000",
  },
  upload: {
    amount: [{ amount: "2500000", denom: "uscrt" }],
    gas: "10000000",
  },
};

async function newClient(mnemonic, rest_endpoint) {
  const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);
  const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);
  const accAddress = pubkeyToAddress(pubkey, "secret");

  console.error(`acc: ${accAddress}`);

  return new SigningCosmWasmClient(
    rest_endpoint,
    accAddress,
    (data) => signingPen.sign(data),
    signingPen.privkey,
    customFees
  );
}

async function getScrtBalance(client) {
  let balanceResponse = await client.getAccount(client.senderAddress);
  const scrtBalanceAfter =
    balanceResponse?.hasOwnProperty("balance") &&
    balanceResponse.balance.length > 0
      ? balanceResponse.balance[0]
      : { amount: 0 };
  return scrtBalanceAfter.amount;
}

module.exports = {
  getFromFaucet,
  fillUpFromFaucet,
  customFees,
  newClient,
  getScrtBalance,
};
