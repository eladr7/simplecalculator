const {
  Secp256k1Pen,
  pubkeyToAddress,
  encodeSecp256k1Pubkey,
  CosmWasmClient,
} = require("secretjs");
const fs = require("fs");

const assert = require("assert");

const {
  getFromFaucet,
  fillUpFromFaucet,
  getScrtBalance,
  newClient,
} = require("./utils.js");

require("dotenv").config();

const PATHS = {
  minter: "artifacts/contracts/minter_contract.wasm",
  lgnd: "artifacts/contracts/snip20.wasm",
  nft: "artifacts/contracts/snip721.wasm",
};

const { Bip39, Random } = require("@iov/crypto");

const createAccount = async (restEndpoint) => {
  // Create random address and mnemonic
  const mnemonic = Bip39.encode(Random.getBytes(16)).toString();

  // This wraps a single keypair and allows for signing.
  const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);

  // Get the public key
  const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);

  // Get the wallet address
  const accAddress = pubkeyToAddress(pubkey, "secret");

  // Query the account
  const client = new CosmWasmClient(restEndpoint);
  const account = await client.getAccount(accAddress);

  console.log("mnemonic: ", mnemonic);
  console.log("address: ", accAddress);
  console.log("account: ", account);

  return [mnemonic, accAddress, account];
};

const sleep = async (ms) => new Promise((r) => setTimeout(r, ms));

const Instantiate = async (client, initMsg, codeId) => {
  const contract = await client.instantiate(
    codeId,
    initMsg,
    "My Counter" + Math.ceil(Math.random() * 10000)
  );
  console.log("contract: ", contract);

  const contractAddress = contract.contractAddress;

  console.log(`Address: ${contractAddress}`);

  return contractAddress;
};

const storeCode = async (path, client) => {
  const wasm = fs.readFileSync(path);
  console.log("Uploading contract");
  const uploadReceipt = await client.upload(wasm, {});
  const codeId = uploadReceipt.codeId;
  console.log("codeId: ", codeId);

  const contractCodeHash = await client.restClient.getCodeHashByCodeId(codeId);
  console.log(`Contract hash: ${contractCodeHash}`);

  return [codeId, contractCodeHash];
};

async function addMinters(secretNetwork, nftContract, minters) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { add_minters: { minters } },
      "",
      []
    );
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(`Gas used for addMinters: ${JSON.stringify(tx2["gas_used"])}`);
  } catch (e) {
    console.log(`Failed to addMinters ${e}`);
  }
  return null;
}

async function mintNfts(secretNetwork, nftContract, amount) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { mint: { amount } },
      "",
      [{ denom: "uscrt", amount: String(1_000_000 * Number(amount)) }]
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for mintNfts (x${amount}): ${JSON.stringify(tx2["gas_used"])}`
    );
    return tx2["gas_used"];
  } catch (e) {
    console.log(`Failed to mint ${e}`);
  }
  return null;
}

async function mintAdminNfts(secretNetwork, nftContract, amount) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { mint_admin: { amount } },
      "",
      []
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for mintAdminNfts (x${amount}): ${JSON.stringify(
        tx2["gas_used"]
      )}`
    );
    return tx2["gas_used"];
  } catch (e) {
    console.log(`Failed to deposit ${e}`);
  }
  return null;
}

async function setTokenAttributes(secretNetwork, nftContract, attributes) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { set_attributes: { tokens: attributes } },
      "",
      []
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for setTokenAttributes: ${JSON.stringify(tx2["gas_used"])}`
    );
  } catch (e) {
    console.log(`Failed to set token attributes ${e}`);
  }
  return null;
}

async function setPlaceholderImage(secretNetwork, nftContract, token_uri) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { set_place_holder: { token_uri } },
      "",
      []
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for setPlaceholderImage: ${JSON.stringify(tx2["gas_used"])}`
    );
  } catch (e) {
    console.log(`Failed to setPlaceholderImage ${e}`);
  }
  return null;
}

async function changeWhitelistLevel(secretNetwork, nftContract, new_level) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { changing_minting_state: { mint_state: new_level } },
      "",
      []
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for changeWhitelistLevel: ${JSON.stringify(tx2["gas_used"])}`
    );
  } catch (e) {
    console.log(`Failed to changeWhitelistLevel ${e}`);
  }
  return null;
}

async function addToWhitelist(secretNetwork, nftContract, address) {
  try {
    let tx = await secretNetwork.execute(
      nftContract,
      { add_whitelist: { addresses: [{ address, amount: 3 }] } },
      "",
      []
    );
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for changeWhitelistLevel: ${JSON.stringify(tx2["gas_used"])}`
    );
  } catch (e) {
    console.log(`Failed to changeWhitelistLevel ${e}`);
  }
  return null;
}

async function viewTokens(secretNetwork, nftContract, address) {
  try {
    return await secretNetwork.queryContractSmart(nftContract, {
      tokens: { owner: address },
    });
  } catch (e) {
    console.log(`Failed to viewTokens ${e}`);
  }
  return null;
}

async function isWhitelisted(secretNetwork, nftContract, address) {
  try {
    return await secretNetwork.queryContractSmart(nftContract, {
      is_whitelisted: { address },
    });
  } catch (e) {
    console.log(`Failed to viewTokens ${e}`);
  }
  return null;
}

let test_suite = async () => {
  // await sleep(5000);

  // await compileAndOptimizeContract(); // cargo wasm, ....
  // await uploadContract();
  // const accountAddress = await createAccount(); // maybe two? one to upload the contract and one to perform actions
  // await fillUpFromFaucet(accountAddress);

  // const balance = 1000;
  // await assertAccountBalance(accountAddress, balance);

  // const contractAddress = await instantiateContract(); // gets the address using: secretcli query compute list-contract-by-code 1

  // const result = await tryAdd(2, 3); // secretcli tx compute execute $CONTRACT '{"try_add": {"n1": 2, "n2": 3}}' --from a --keyring-backend test

  // const vk = await generateViewingKey(accountAddress);

  // const history = await getCalculationsHistory(accountAddress, vk);

  let REST_ENDPOINT = process.env.SECRET_GRPC_WEB_URL;

  const secretNetwork = await newClient(process.env.MNEMONIC, REST_ENDPOINT);

  const chainId = await secretNetwork.getChainId();
  console.log("chainId: ", chainId);

  const height = await secretNetwork.getHeight();
  console.log("height: ", height);

  if (chainId === "secretdev-1") {
    await fillUpFromFaucet(secretNetwork, 10_000_000);
  }

  //console.log(`validator address: ${validatorAddress}`);

  const [mintContractCode, mintContractHash] = await storeCode(
    PATHS.minter,
    secretNetwork
  );
  const [gpNftCode, gpNftHash] = await storeCode(PATHS.nft, secretNetwork);
  const [baconCode, baconContractHash] = await storeCode(
    PATHS.lgnd,
    secretNetwork
  );

  const nftInitMsg = {
    name: "GpigsTest",
    entropy: "YWE",
    revealer: secretNetwork.senderAddress,
    symbol: "gpcc",
    royalty_info: {
      decimal_places_in_rates: 3,
      royalties: [{ recipient: secretNetwork.senderAddress, rate: 50 }],
    },
  };

  const nftContractAddress = await Instantiate(
    secretNetwork,
    nftInitMsg,
    gpNftCode
  );

  const lgndInitMsg = {
    prng_seed: "YWE",
    symbol: "BACON",
    name: "bacon",
    decimals: 6,
    initial_balances: [
      { address: secretNetwork.senderAddress, amount: "10000000000" },
    ],
    config: {
      public_total_supply: true,
      enable_deposit: false,
      enable_redeem: false,
      enable_mint: true,
      enable_burn: true,
    },
  };

  const lgndToken = await Instantiate(secretNetwork, lgndInitMsg, baconCode);

  const mintingContractInitMsg = {
    nft_count: 100,
    nft_contract: { address: nftContractAddress, hash: gpNftHash },
    bacon_contract: { address: lgndToken, hash: baconContractHash },
    random_seed: "YWE",
    price: "1000000",
    whitelist_price: "100000",
  };
  const mintingContractAddress = await Instantiate(
    secretNetwork,
    mintingContractInitMsg,
    mintContractCode
  );

  await addMinters(secretNetwork, nftContractAddress, [mintingContractAddress]);
  await addMinters(secretNetwork, lgndToken, [mintingContractAddress]);

  await setPlaceholderImage(
    secretNetwork,
    mintingContractAddress,
    "https://variety.com/wp-content/uploads/2018/07/overwatch-loot-box.jpg"
  );

  const attributes = [];

  for (let i = 0; i < 100; i++) {
    attributes.push({
      token_id: i.toString(10),
      attributes: {
        custom_traits: [{ trait_type: "penis_length", value: `${i}` }],
        rarity: 0,
        token_uri: "https://data.whicdn.com/images/311555755/original.jpg",
      },
    });
  }

  await setTokenAttributes(secretNetwork, mintingContractAddress, attributes);

  let result = await mintAdminNfts(secretNetwork, mintingContractAddress, 2);
  assert(result !== null, "failed to mint as admin");

  let tokens = await viewTokens(
    secretNetwork,
    nftContractAddress,
    secretNetwork.senderAddress
  );
  console.log(`minted nfts: ${JSON.stringify(tokens)}`);

  console.log(`Testing failure to mint`);
  result = await mintNfts(secretNetwork, mintingContractAddress, 2);
  assert(
    result === null,
    "succeeded to mint even though minting not enabled yet"
  );

  console.log(`enabling whitelisted minting`);
  await changeWhitelistLevel(secretNetwork, mintingContractAddress, 2);

  let [mnemonic, accAddress, __] = await createAccount(REST_ENDPOINT);
  let userNetwork = await newClient(mnemonic, REST_ENDPOINT);
  await addToWhitelist(secretNetwork, mintingContractAddress, accAddress);

  result = await isWhitelisted(
    secretNetwork,
    mintingContractAddress,
    accAddress
  );
  console.log(`Is user address whitelisted? ${JSON.stringify(result)}`);

  const DEPOSIT_AMOUNT = 10_000_000;

  await secretNetwork.sendTokens(
    accAddress,
    [{ amount: String(DEPOSIT_AMOUNT), denom: "uscrt" }],
    "",
    {
      amount: [{ amount: "50000", denom: "uscrt" }],
      gas: "200000",
    }
  );

  console.log(`\tsent 10scrt from main account to user`);

  // mint 2
  result = await mintNfts(userNetwork, mintingContractAddress, 2);
  assert(result !== null, "failed to mint as whitelisted");
  tokens = await viewTokens(userNetwork, nftContractAddress, accAddress);
  console.log(`minted nfts: ${JSON.stringify(tokens)}`);

  result = await isWhitelisted(
    secretNetwork,
    mintingContractAddress,
    accAddress
  );
  console.log(`Is user address whitelisted? ${JSON.stringify(result)}`);

  // mint another 1
  result = await mintNfts(userNetwork, mintingContractAddress, 1);
  assert(result !== null, "failed to mint as whitelisted");
  tokens = await viewTokens(userNetwork, nftContractAddress, accAddress);
  console.log(`minted nfts: ${JSON.stringify(tokens)}`);

  result = await isWhitelisted(
    secretNetwork,
    mintingContractAddress,
    accAddress
  );
  console.log(`Is user address whitelisted? ${JSON.stringify(result)}`);

  // mint another 1 - should fail
  result = await mintNfts(userNetwork, mintingContractAddress, 1);
  assert(
    result === null,
    "succeeded to mint even though whitelisted address should have no more allocation"
  );

  // change whitelist level
  console.log(`changing whitelist level to public`);
  await changeWhitelistLevel(secretNetwork, mintingContractAddress, 3);
  [_, accAddress, __] = await createAccount(REST_ENDPOINT);
  userNetwork = await newClient(mnemonic, REST_ENDPOINT);

  await secretNetwork.sendTokens(
    accAddress,
    [{ amount: String(DEPOSIT_AMOUNT), denom: "uscrt" }],
    "",
    {
      amount: [{ amount: "50000", denom: "uscrt" }],
      gas: "200000",
    }
  );

  console.log(`\tsent 10scrt from main account to user`);

  // public mint
  result = await mintNfts(userNetwork, mintingContractAddress, 1);
  assert(result !== null, "failed to mint after changing whitelist level");
  tokens = await viewTokens(userNetwork, nftContractAddress, accAddress);
  console.log(`minted nfts: ${JSON.stringify(tokens)}`);

  // still todo:
  // reveal
  // test

  try {
  } catch (e) {
    console.log(e);
  }
};

describe("minting", function () {
  this._timeout = 1000000000;

  it("all works", test_suite);
});
