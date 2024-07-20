//@ts-ignore
import { Deferred, deferred } from "https://deno.land/std@0.158.0/async/mod.ts";
// import { RpcTransport } from "./http.transport";

export interface Option {
  protocols?: string | string[];
  binaryType: "arraybuffer" | "blob";
}

export class Socket {
  // deno-lint-ignore no-explicit-any
  #reader: ReadableStreamDefaultReader<MessageEvent<any>>;

  static async connect(
    url: string | URL,
    opt: Option = { binaryType: "blob" },
  ) {
    const ws = new WebSocket(url, opt.protocols);
    ws.binaryType = opt.binaryType;
    const closed = deferred<CloseEvent>();

    const readableStream: ReadableStream = await new Promise(
      (resolve, reject) => {
        ws.onerror = reject;
        ws.onopen = () => {
          resolve(
            new ReadableStream({
              start(controler) {
                ws.onmessage = (msg) => controler.enqueue(msg);
                ws.onerror = (err) => controler.error(err);
                ws.onclose = (ev) => {
                  closed.resolve(ev);
                  controler.close();
                };
              },
            }),
          );
        };
      },
    );
    return new Socket(ws, closed, readableStream);
  }

  constructor(
    private ws: WebSocket,
    public readonly closed: Deferred<CloseEvent>,
    stream: ReadableStream,
  ) {
    this.#reader = stream.getReader();
    closed.then(() => {
      if (stream.locked) this.#reader.releaseLock();
    });
  }

  get bufferedAmount() {
    return this.ws.bufferedAmount;
  }

  send(data: string | ArrayBufferLike | Blob | ArrayBufferView) {
    this.ws.send(data);
  }

  read() {
    return this.#reader.read();
  }

  async *[Symbol.asyncIterator]() {
    while (true) {
      const { done, value } = await this.read();
      if (done) return value;
      yield value;
    }
  }

  close(code?: number, reason?: string) {
    this.ws.close(code, reason);
  }
}

// export class WebSocketTransport implements RpcTransport {
//     static async connect(url: string) {
//         const socket = await Socket.connect(url, { binaryType: "arraybuffer" });
//         return new WebSocketTransport(socket)
//     }
//     constructor(private socket: Socket) { }
//     unary() {
//         return {
//             flush() { /* noop */ },
//             write: (bytes: Uint8Array) => this.socket.send(bytes),
//             output: async () => {
//                 try {
//                     const { done, value } = await this.socket.read();
//                     if (done || !(value.data instanceof ArrayBuffer)) {
//                         throw new Error("Invalid data type: " + typeof value?.data);
//                     }
//                     return value.data;
//                 } catch (error) {
//                     this.socket.close()
//                     throw error
//                 }
//             }
//         }
//     }
//     async close() {
//         this.socket.close()
//     }
// }
