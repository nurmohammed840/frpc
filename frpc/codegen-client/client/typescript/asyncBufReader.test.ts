import { assert, assertEquals, assertRejects } from "https://deno.land/std@0.175.0/testing/asserts.ts";
import { AsyncBufReader } from "./http.transport.ts";

function s(i: () => AsyncIterable<Uint8Array>) {
    return new AsyncBufReader(ReadableStream.from(i()).getReader())
}

Deno.test("test async buf reader (read)", async () => {
    let reader = s(async function* () {
        yield Uint8Array.from([0, 1, 2, 3, 4])
    })
    assertEquals(await reader.read(2), Uint8Array.from([0, 1]));
    assertEquals(await reader.read(0), Uint8Array.from([]));
    assertEquals(await reader.read(1), Uint8Array.from([2]));
    assertEquals(await reader.read(3), Uint8Array.from([3, 4]));
    assertEquals(reader.data, Uint8Array.from([]));
    await assertRejects(() => reader.read(0))

    reader = s(async function* () {
        yield Uint8Array.from([])
    })
    assertEquals(await reader.read(1), Uint8Array.from([]));
})

Deno.test("test async buf reader (errors)", async () => {
    let reader = s(async function* () { })
    await assertRejects(() => reader.read(0))

    reader = s(async function* () {
        yield Uint8Array.from([1, 2])
        yield Uint8Array.from([])
    })
    try {
        await reader.readExact(3)
        throw "Unreachable"
    } catch (error) {
        assertEquals(error?.message, "failed to fill whole buffer")
    }

    reader = s(async function* () {
        yield Uint8Array.from([1, 2])
    })
    try {
        await reader.readExact(3)
        throw "Unreachable"
    } catch (error) {
        assertEquals(error?.message, "unexpected EOF")
    }

})

Deno.test("test async buf reader (readExact)", async () => {
    let reader = s(async function* () {
        yield Uint8Array.from([])
        yield Uint8Array.from([1, 2, 3])
        yield Uint8Array.from([1, 2])
        yield Uint8Array.from([3, 4, 5])
        yield Uint8Array.from([6, 7])
    })
    assertEquals(await reader.readExact(3), Uint8Array.from([1, 2, 3]));
    assertEquals(reader.data, Uint8Array.from([]));
    assertEquals(await reader.readExact(6), Uint8Array.from([1, 2, 3, 4, 5, 6]));
    assertEquals(reader.data, Uint8Array.from([7]));

    reader = s(async function* () {
        yield Uint8Array.from([1, 2, 3])
        yield Uint8Array.from([4, 5])
        yield Uint8Array.from([6, 7, 8])
        yield Uint8Array.from([9, 10])
    });

    let read_1 = await reader.readExact(4);
    assertEquals(reader.data, Uint8Array.from([5]));
    let read_2 = await reader.readExact(4);
    assertEquals(reader.data, Uint8Array.from([]));
    let read_3 = await reader.readExact(2);
    assertEquals(reader.data, Uint8Array.from([]));

    assert(read_3.buffer === reader.data.buffer);
    assertEquals(read_3, Uint8Array.from([9, 10]));
    assertEquals(read_2, Uint8Array.from([5, 6, 7, 8]));
    assertEquals(read_1, Uint8Array.from([1, 2, 3, 4]));

    reader = s(async function* () {
        yield Uint8Array.from([1, 2])
        yield Uint8Array.from([3, 4])
    });
    let buf = await reader.readExact(3);
    assert(buf.buffer !== reader.data.buffer);
    assertEquals(buf, Uint8Array.from([1, 2, 3]));
    assertEquals(reader.data, Uint8Array.from([4]));
})

