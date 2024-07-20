import { assertEquals } from "https://deno.land/std@0.175.0/testing/asserts.ts";
import { BufWriter, Decoder, Option, Result, Write } from "./databuf.ts";

class DefaultWriter implements Write {
  bytes: number[] = [];
  write(bytes: Uint8Array) {
    this.bytes = this.bytes.concat(...bytes);
  }
  flush() {}
}

Deno.test("Serde test: var int", () => {
  const writer = new DefaultWriter();
  const encoder = new BufWriter(writer);

  encoder.len_u15(0); // [0]
  encoder.len_u15(127); // [127]
  encoder.len_u15(128); // [128, 128]
  encoder.len_u15(32767); // [255, 255]

  encoder.len_u30(0); // [0]
  encoder.len_u30(63); // [63]
  encoder.len_u30(64); // [64, 64]
  encoder.len_u30(16383); // [127, 255]
  encoder.len_u30(16384); // [128, 64, 0]
  encoder.len_u30(4194303); // [191, 255, 255]
  encoder.len_u30(4194304); // [192, 64, 0, 0]
  encoder.len_u30(1073741823); // [255, 255, 255, 255]

  encoder.flush();
  assertEquals(writer.bytes, [
    0,
    127,
    128,
    128,
    255,
    255,
    0,
    63,
    64,
    64,
    127,
    255,
    128,
    64,
    0,
    191,
    255,
    255,
    192,
    64,
    0,
    0,
    255,
    255,
    255,
    255,
  ]);

  const decoder = Decoder.from(new Uint8Array(writer.bytes));
  assertEquals(decoder.len_u15(), 0);
  assertEquals(decoder.len_u15(), 127);
  assertEquals(decoder.len_u15(), 128);
  assertEquals(decoder.len_u15(), 32767);

  assertEquals(decoder.len_u30(), 0);
  assertEquals(decoder.len_u30(), 63);
  assertEquals(decoder.len_u30(), 64);
  assertEquals(decoder.len_u30(), 16383);
  assertEquals(decoder.len_u30(), 16384);
  assertEquals(decoder.len_u30(), 4194303);
  assertEquals(decoder.len_u30(), 4194304);
  assertEquals(decoder.len_u30(), 1073741823);
});

Deno.test("Serde test: LEB128", () => {
  const writer = new DefaultWriter();
  const encoder = new BufWriter(writer);

  const I16_MIN = -32768;
  const I32_MIN = -2147483648;
  const I64_MIN = -9223372036854775808n;

  const U16_MAX = Math.pow(2, 16) - 1;
  const U32_MAX = Math.pow(2, 32) - 1;
  const U64_MAX = (1n << 64n) - 1n;
  const U128_MAX = (1n << 128n) - 1n;

  encoder.num("U", 16)(0); // [0]
  encoder.num("U", 16)(U16_MAX); // [255, 255, 3]
  encoder.num("U", 32)(U32_MAX); // [255, 255, 255, 255, 15]

  encoder.flush();
  assertEquals(writer.bytes, [0, 255, 255, 3, 255, 255, 255, 255, 15]);

  encoder.num("U", 64)(U64_MAX);
  encoder.num("U", 128)(U128_MAX);

  encoder.num("I", 16)(I16_MIN);
  encoder.num("I", 32)(I32_MIN);
  encoder.num("I", 64)(I64_MIN);
  encoder.flush();

  const decoder = Decoder.from(new Uint8Array(writer.bytes));
  assertEquals(decoder.num("U", 16)(), 0);
  assertEquals(decoder.num("U", 16)(), U16_MAX);
  assertEquals(decoder.num("U", 32)(), U32_MAX);
  assertEquals(decoder.num("U", 64)(), U64_MAX);
  assertEquals(decoder.num("U", 128)(), U128_MAX);

  assertEquals(decoder.num("I", 16)(), I16_MIN);
  assertEquals(decoder.num("I", 32)(), I32_MIN);
  assertEquals(decoder.num("I", 64)(), I64_MIN);
});

Deno.test("Serde test: string, bytes", () => {
  const writer = new DefaultWriter();
  const encoder = new BufWriter(writer);

  encoder.str("Hello, World!");
  encoder.arr(encoder.u8)([42, 24]);

  encoder.flush();
  const decoder = Decoder.from(new Uint8Array(writer.bytes));

  assertEquals(decoder.str(), "Hello, World!");
  assertEquals(decoder.arr(decoder.u8)(), [42, 24]);
});

Deno.test("Serde test: common type", () => {
  const bool = true;

  const some = { value: "some" };
  const none: Option<string> = { value: null };

  type Vec2d = [number, number];
  const ok: Result<Vec2d, string> = { type: "Ok", value: [4, 2] };
  const err: Result<string, Vec2d> = { type: "Err", value: [2, 4] };

  const writer = new DefaultWriter();
  const encoder = new BufWriter(writer);

  encoder.bool(bool);

  encoder.option(encoder.str)(some);
  encoder.option(encoder.str)(none);

  encoder.result(encoder.fixed_arr(encoder.u8), encoder.str)(ok);
  encoder.result(encoder.str, encoder.fixed_arr(encoder.u8))(err);

  encoder.flush();
  const decoder = Decoder.from(new Uint8Array(writer.bytes));

  assertEquals(decoder.bool(), bool);

  assertEquals(decoder.option(decoder.str)(), some);
  assertEquals(decoder.option(decoder.str)(), none);

  assertEquals(
    decoder.result(decoder.fixed_arr(decoder.u8, 2), decoder.str)(),
    ok,
  );
  assertEquals(
    decoder.result(decoder.str, decoder.fixed_arr(decoder.u8, 2))(),
    err,
  );
});

Deno.test("Serde test: Complex type", () => {
  const value: Map<number, Option<string>> = new Map();
  value.set(0, { value: null });
  value.set(1, { value: "some" });

  const ok = { type: "Ok" as const, value };
  const err = {
    type: "Err" as const,
    value: ["Error: 1", "Error: 2"],
  };

  const writer = new DefaultWriter();
  const e = new BufWriter(writer);

  const encode = e.result(e.map(e.u8, e.option(e.str)), e.arr(e.str));
  encode(ok);
  encode(err);

  e.flush();
  const d = Decoder.from(new Uint8Array(writer.bytes));

  const decode = d.result(d.map(d.u8, d.option(d.str)), d.arr(d.str));
  assertEquals(decode(), ok);
  assertEquals(decode(), err);
});
