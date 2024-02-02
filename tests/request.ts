import { JsonRpcProvider } from "ethers";
const host = "http://127.0.0.1:3001";
const testHost = "https://bsc.meowrpc.com";

const testProvider = new JsonRpcProvider(testHost, undefined, {
  cacheTimeout: -1,
});
async function main() {
  for (const chainId of (Bun.env.SUPPORTED_CHAIN_IDS as string).split(",")) {
    const provider = new JsonRpcProvider(
      `${host}/v1/chain/${chainId}`,
      undefined,
      {
        cacheTimeout: -1,
      },
    );
    for (let i = 0; i < 100; i++) {
      console.log(i);
      const blockNumber = await provider.getBlockNumber();
      console.log({ blockNumber });
      Bun.sleepSync(100);
    }
  }
}

async function f() {
  for (let i = 0; i < 10; i++) {
    main();
  }
}

await f();
