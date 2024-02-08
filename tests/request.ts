import { JsonRpcProvider } from "ethers";
const host = "http://127.0.0.1:3001";
const testHost = "https://bsc.meowrpc.com";
const apiKey = "72320785-f772-47e3-a9e3-569a26f75c1f"
const url = `http://localhost:3001/v2/chain/56/${apiKey}`

const testProvider = new JsonRpcProvider(testHost, undefined, {
  cacheTimeout: -1,
});
async function main() {
  for (const chainId of (Bun.env.SUPPORTED_CHAIN_IDS as string).split(",")) {
    const provider = new JsonRpcProvider(
      url,
      undefined,
      {
        cacheTimeout: -1,
      },
    );
    for (let i = 0; i < 100; i++) {
      console.log(i);
      try {

    const a = await  provider.getBlockNumber() // .catch(console.error)
    console.log(a)
      } catch (err) {
        console.error(err)
      }
    }
  }
}

async function f() {
    main();
}

await f();
