#!/usr/bin/env -S deno run --allow-net="localhost" --unsafely-ignore-certificate-errors="localhost"

import { HttpTransport } from "../out/http.transport.ts";

import SSE from "../out/ServerSentEvents.ts";
import Stateful from "../out/Stateful.ts";
import Greeter from "../out/Greeter.ts";

let greeter = new Greeter(new HttpTransport("https://localhost:4433/greeter"));
console.log(await greeter.SayHello({ name: "Nur!" }));
console.log(await greeter.SayHelloAgain({ name: "Mo!" }));


let sse = new SSE(new HttpTransport("https://localhost:4433/sse"));
for await (const msg of sse.say_hello_many_times(3, 300)) {
    console.log(msg);
}


let server = new Stateful(new HttpTransport("https://localhost:4433/stateful"));
console.log(await server.whats_my_name());
console.log(await server.my_name_is("Nur"));
console.log(await server.whats_my_name());
