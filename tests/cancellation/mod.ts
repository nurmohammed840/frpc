#!/usr/bin/env -S deno run --allow-net="localhost" --unsafely-ignore-certificate-errors="localhost"
import { assertRejects, assertEquals } from "https://deno.land/std@0.175.0/testing/asserts.ts";
import { HttpTransport } from "../../target/rpc/http.transport.ts";
import Cancellation from "../../target/rpc/Cancellation.ts";

let lib = new Cancellation(new HttpTransport("https://localhost:4433/rpc/cancellation"));
let deadline = 2;

async function make_unary_call(timeout: number) {
    let req = lib.unary_call(deadline);
    setTimeout(() => req.abort("TimeOut!"), timeout * 1000);
    assertEquals(await req, { type: "Err", value: "deadline passed!" });
}

async function make_stream_rpc_call(timeout: number) {
    let stream = lib.stream(deadline);
    setTimeout(() => stream.abort("TimeOut!"), timeout * 1000);
    assertEquals((await stream.next()).value, `waiting for ${deadline} secs`);
    assertEquals((await stream.next()).value, "timeout!");
    assertEquals((await stream.next()).value, { type: "Err", value: "deadline passed!" });
}

await Promise.all([
    make_unary_call(3),
    assertRejects(() => make_unary_call(1)),

    make_stream_rpc_call(3),
    assertRejects(() => make_stream_rpc_call(1)),
])