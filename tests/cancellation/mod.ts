#!/usr/bin/env -S deno run --allow-net="localhost" --unsafely-ignore-certificate-errors="localhost"
import { assertEquals } from "https://deno.land/std@0.175.0/testing/asserts.ts";
import { HttpTransport } from "../../target/rpc/http.transport.ts";
import Cancellation from "../../target/rpc/Cancellation.ts";

let lib = new Cancellation(new HttpTransport("https://localhost:4433/rpc/cancellation"));


async function test_cancelation() {
    try {
        let abortController = new AbortController();
        setTimeout(() => abortController.abort("TimeOut!"), 2 * 1000);
        await lib.sleep_for_eternity()({ signal: abortController.signal });
        throw "Done"
    } catch (error) {
        assertEquals(error, "TimeOut!")
    }
}

async function test_stream_cancelation() {
    try {
        let abortController = new AbortController();
        setTimeout(() => abortController.abort("TimeOut!"), 3 * 1000);
        let events = lib.stream_sleep_for_eternity()({ signal: abortController.signal });

        assertEquals((await events.next()).value, "Going to sleep for eternity");
        await events.next();
        throw "Done"
    } catch (error) {
        assertEquals(error, "TimeOut!")
    }
}

await Promise.all([
    test_cancelation(),
    test_stream_cancelation()
])
