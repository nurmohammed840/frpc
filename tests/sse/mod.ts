#!/usr/bin/env -S deno run --allow-net="localhost" --unsafely-ignore-certificate-errors="localhost"

import {
  assert,
  assertEquals,
} from "https://deno.land/std@0.175.0/testing/asserts.ts";
import { HttpTransport } from "../../target/rpc/http.transport.ts";
import Lib from "../../target/rpc/SSETest.ts";

let lib = new Lib(new HttpTransport("https://localhost:4433/rpc/sse"));

{
  let buffers = lib.buffers()();
  for (let i = 1;; i++) {
    let { value, done } = await buffers.next();
    if (done) {
      assertEquals(value, "Done!");
      break;
    }
    let u8 = new Uint8Array([i])[0];
    assert((value as Uint8Array).every((v) => v == u8));
  }
}

async function test_chunks(iter: number, size: number) {
  let chunks = lib.chunks(iter, size)();
  let i = 0;
  for await (const chunk of chunks) {
    assert(chunk.every((v) => v == i));
    i++;
  }
  assertEquals(i, iter);
}

await Promise.all([
  test_chunks(3, 8 * 1024), // 8kb
  test_chunks(3, 512 * 1024), // 512kb
  test_chunks(3, 4 * 1024 * 1024), // 4MB
]);
