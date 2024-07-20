#!/usr/bin/env -S deno run --allow-net=localhost --unsafely-ignore-certificate-errors=localhost

import { assertEquals, assertThrows } from "https://deno.land/std@0.175.0/testing/asserts.ts";
import { HttpTransport } from "../../target/rpc/http.transport.ts";
import Lib, { Log } from "../../target/rpc/EchoTest.ts";

let lib = new Lib(new HttpTransport("https://localhost:4433/rpc/echo"));

let MAX_U8 = (1 << 8) - 1
let MAX_U16 = (1 << 16) - 1
let MAX_U32 = Number((1n << 32n) - 1n)
let MAX_U64 = (1n << 64n) - 1n
let MAX_U128 = (1n << 128n) - 1n

let MIN_I8 = -(1 << 8 - 1);
let MIN_I16 = -(1 << 16 - 1);
let MIN_I32 = Number(-(1n << 32n - 1n))
let MIN_I64 = -(1n << 64n - 1n)
let MIN_I128 = -(1n << 128n - 1n)

let MAX_I8 = -MIN_I8 - 1;
let MAX_I16 = -MIN_I16 - 1;
let MAX_I32 = -MIN_I32 - 1
let MAX_I64 = -MIN_I64 - 1n
let MAX_I128 = -MIN_I128 - 1n

await lib.log(Log.Disable)();

// -------------------------------------------------------

assertEquals(0, await lib.echo_u8(0)());
assertEquals(0, await lib.echo_u16(0)());
assertEquals(0, await lib.echo_u32(0)());
assertEquals(0n, await lib.echo_u64(0n)());
assertEquals(0n, await lib.echo_u128(0n)());

assertEquals(MAX_U8, await lib.echo_u8(MAX_U8)());
assertEquals(MAX_U16, await lib.echo_u16(MAX_U16)());
assertEquals(MAX_U32, await lib.echo_u32(MAX_U32)());
assertEquals(MAX_U64, await lib.echo_u64(MAX_U64)());
assertEquals(MAX_U128, await lib.echo_u128(MAX_U128)());

assertThrows(lib.echo_u8(-1));
assertThrows(lib.echo_u16(-1));
assertThrows(lib.echo_u32(-1));
assertThrows(lib.echo_u64(-1n));
assertThrows(lib.echo_u128(-1n));

assertThrows(lib.echo_u8(MAX_U8 + 1));
assertThrows(lib.echo_u16(MAX_U16 + 1));
assertThrows(lib.echo_u32(MAX_U32 + 1));
assertThrows(lib.echo_u64(MAX_U64 + 1n));
assertThrows(lib.echo_u128(MAX_U128 + 1n));

// -------------------------------------------------------

assertEquals(MIN_I8, await lib.echo_i8(MIN_I8)());
assertEquals(MIN_I16, await lib.echo_i16(MIN_I16)());
assertEquals(MIN_I32, await lib.echo_i32(MIN_I32)());
assertEquals(MIN_I64, await lib.echo_i64(MIN_I64)());
assertEquals(MIN_I128, await lib.echo_i128(MIN_I128)());

assertEquals(MAX_I8, await lib.echo_i8(MAX_I8)());
assertEquals(MAX_I16, await lib.echo_i16(MAX_I16)());
assertEquals(MAX_I32, await lib.echo_i32(MAX_I32)());
assertEquals(MAX_I64, await lib.echo_i64(MAX_I64)());
assertEquals(MAX_I128, await lib.echo_i128(MAX_I128)());

assertThrows(lib.echo_i8(MIN_I8 - 1));
assertThrows(lib.echo_i16(MIN_I16 - 1));
assertThrows(lib.echo_i32(MIN_I32 - 1));
assertThrows(lib.echo_i64(MIN_I64 - 1n));
assertThrows(lib.echo_i128(MIN_I128 - 1n));

assertThrows(lib.echo_i8(MAX_I8 + 1));
assertThrows(lib.echo_i16(MAX_I16 + 1));
assertThrows(lib.echo_i32(MAX_I32 + 1));
assertThrows(lib.echo_i64(MAX_I64 + 1n));
assertThrows(lib.echo_i128(MAX_I128 + 1n));

// -------------------------------------------------------

let option = { value: null };
assertEquals(option, await lib.echo_option(option)());

let option1 = { value: { value: null } };
assertEquals(option1, await lib.echo_option(option1)());

let option2 = { value: { value: "data" } };
assertEquals(option2, await lib.echo_option(option2)());

// -------------------------------------------------------

let ok = { type: "Ok" as const, value: "Output" };
assertEquals(ok, await lib.echo_result(ok)());

let err = { type: "Err" as const, value: "Error" };
assertEquals(err, await lib.echo_result(err)());

// -------------------------------------------------------

assertEquals(true, await lib.echo_bool(true)());
assertEquals(false, await lib.echo_bool(false)());
assertEquals("Hello World!", await lib.echo_str("Hello World!")());

// -------------------------------------------------------

let map = new Map([["2", 2], ["1", 1], ["3", 3]]);
assertEquals(map, await lib.echo_map(map)());
assertEquals(
    [["1", 1], ["2", 2], ["3", 3]],
    [...(await lib.echo_sorted_map(map)()).entries()]
);

// -------------------------------------------------------

let bufs = {
    vec_2d: Float32Array.from([1.2, 2.3]),
    vec_3d: Float64Array.from([3.4, 4.5, 5.6]),
    floats: Float32Array.from([42]),
    long_floats: Float64Array.from([42]),
    sorted_nums: Int8Array.from([2, 0, 1, -1]),
    big_nums: [42n, BigInt(Number.MAX_SAFE_INTEGER) * 2n],
    bytes: Uint8Array.from([1, 2, 3]),
}

let echo_bufs = await lib.echo_bufs(bufs)();
bufs.sorted_nums = Int8Array.from([-1, 0, 1, 2]);
assertEquals(bufs, echo_bufs);
