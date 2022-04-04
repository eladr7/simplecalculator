const {
  Secp256k1Pen,
  pubkeyToAddress,
  encodeSecp256k1Pubkey,
  CosmWasmClient,
} = require("secretjs");
const zlib = require("zlib");
const fs = require("fs");

const assert = require("assert");

const {
  getFromFaucet,
  fillUpFromFaucet,
  getScrtBalance,
  newClient,
} = require("../tests/utils.js");

require("dotenv").config();

const PATHS = {
  // contract: "./contract.wasm.gz", //Elad: extract contract.wasm.gz to contract.wasm
  contract: "./contract.wasm", //Elad: extract contract.wasm.gz to contract.wasm
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
    "My Simple calculator" + Math.ceil(Math.random() * 10000)
  );
  console.log("contract: ", contract);

  const contractAddress = contract.contractAddress;

  console.log(`Address: ${contractAddress}`);

  return contractAddress;
};

const storeCode = async (path, client) => {
  const wasm = fs.readFileSync(path);
  // const wasm = zlib.gunzipSync(fs.readFileSync(path));
  console.log("Uploading contract");
  const uploadReceipt = await client.upload(wasm, {});
  const codeId = uploadReceipt.codeId;
  console.log("codeId: ", codeId);

  const contractCodeHash = await client.restClient.getCodeHashByCodeId(codeId);
  console.log(`Contract hash: ${contractCodeHash}`);

  return [codeId, contractCodeHash];
};

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

async function add(secretNetwork, contract, num1, num2) {
  try {
    let tx = await secretNetwork.execute(
      contract,
      { add: { n1: num1, n2: num2 } },
      "",
      []
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(`Gas used for add: ${JSON.stringify(tx2["gas_used"])}`);
  } catch (e) {
    console.log(`Failed to add numbers ${e}`);
  }
  return null;
}

async function generateViewingKey(secretNetwork, contract, entropyString) {
  try {
    let tx = await secretNetwork.execute(
      contract,
      { generate_viewing_key: { entropy: entropyString } },
      "",
      []
    );
    await sleep(6000);
    let tx2 = await secretNetwork.restClient.txById(tx["transactionHash"]);
    console.log(
      `Gas used for generateViewingKey: ${JSON.stringify(tx2["gas_used"])}`
    );
  } catch (e) {
    console.log(`Failed to generate a viewing key ${e}`);
  }
  return null;
}

async function queryCalculationsHistory(
  secretNetwork,
  contract,
  accAddress,
  vk,
  stepBack = null
) {
  try {
    return await secretNetwork.queryContractSmart(contract, {
      get_history: { address: accAddress, key: vk, steps_back: stepBack },
    });
  } catch (e) {
    console.log(`Failed to viewTokens ${e}`);
  }
  return null;
}

let test_suite = async () => {
  // await sleep(5000);

  // Elad:: compile the contract in the yml:
  // - name: Compile the contract to a .wasm.gz file
  // run: cargo wasm && docker run --rm -v "$(pwd)":/contract --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry enigmampc/secret-contract-optimizer:1.0.6

  let REST_ENDPOINT = process.env.SECRET_GRPC_WEB_URL;

  const secretNetwork = await newClient(process.env.MNEMONIC, REST_ENDPOINT);

  const chainId = await secretNetwork.getChainId();
  console.log("chainId: ", chainId);

  const height = await secretNetwork.getHeight();
  console.log("height: ", height);

  if (chainId === "pulsar-2") {
    await fillUpFromFaucet(secretNetwork, 10_000_000);
  }

  const [contractCode, contractHash] = await storeCode(
    PATHS.contract,
    secretNetwork
  );
  console.log(`contractHash is : ${contractHash}`);

  const initMsg = {
    prng_seed: "waehfjklasd",
  };
  const contractAddress = await Instantiate(
    secretNetwork,
    initMsg,
    contractCode
  );
  console.log(`Contract address is : ${contractAddress}`);

  let [mnemonic, accAddress, __] = await createAccount(REST_ENDPOINT);
  let userNetwork = await newClient(mnemonic, REST_ENDPOINT);

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

  // tokens = await viewTokens(userNetwork, contractAddress, accAddress);
  // console.log(`Account balance: ${JSON.stringify(tokens)}`);

  // const result = await add(secretNetwork, contractAddress, 2, 3); // secretcli tx compute execute $CONTRACT '{"try_add": {"n1": 2, "n2": 3}}' --from a --keyring-backend test
  // const vk = await generateViewingKey(secretNetwork, contractAddress, "dsfsd");

  // const history = await queryCalculationsHistory(
  //   secretNetwork,
  //   contract,
  //   accAddress,
  //   vk
  // );
  // assert(history === "2 + 3 = 5");

  try {
  } catch (e) {
    console.log(e);
  }
};

describe("Arithmetics", function () {
  // this._timeout = 1000000000;
  jest.setTimeout(50000);

  it("all works", test_suite);
});
