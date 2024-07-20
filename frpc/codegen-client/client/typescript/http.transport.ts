export interface Write {
  write(bytes: Uint8Array): void;
  flush(): void;
}

export interface RpcTransport {
  unary(): Write & { call(_?: RequestInit): Promise<Uint8Array> };
  sse(): Write & { call(_?: RequestInit): AsyncGenerator<Uint8Array> };
  close(): Promise<void>;
}

export interface HttpTransportRequestInit
  extends Pick<RequestInit, "mode" | "keepalive" | "headers"> {}

export class HttpTransport implements RpcTransport {
  constructor(
    public url: URL | RequestInfo,
    public option = {
      maxChunkSize: 8 * 1024 * 1024,
      requestInit: {} as HttpTransportRequestInit | undefined,
    },
  ) {}
  unary() {
    const { url, option } = this;
    const chunks: Uint8Array[] = [];
    return {
      write(bytes: Uint8Array) {
        chunks.push(bytes);
      },
      flush() {},
      async call(requestInit: RequestInit) {
        const body = concat_uint8(chunks);
        const res = await fetch(url, {
          ...option.requestInit,
          ...requestInit,
          method: "POST",
          body,
        });
        if (!res.ok) {
          throw new Error("Bad request");
        }
        return new Uint8Array(await res.arrayBuffer());
      },
    };
  }

  sse() {
    const { url, option } = this;
    const chunks: Uint8Array[] = [];
    return {
      write(bytes: Uint8Array) {
        chunks.push(bytes);
      },
      flush() {},

      async *call(requestInit: RequestInit) {
        const body = concat_uint8(chunks);
        const res = await fetch(url, {
          ...option.requestInit,
          ...requestInit,
          method: "POST",
          body,
        });
        if (!res.body) {
          throw new Error("unexpected empty body");
        }
        let reader = new AsyncBufReader(res.body.getReader());

        while (true) {
          let head = await reader.readExact(4);
          let fin = (head[3] & 0b1000_0000) == 0b1000_0000;
          head[3] &= 0b0111_1111;
          let len = new DataView(head.buffer).getUint32(0, true);
          if (fin && len == 0) {
            return new Uint8Array(0);
          }
          if (len > option.maxChunkSize) {
            throw new Error(
              `Max chunk size is ${option.maxChunkSize}, But actual size is ${len} bytes`,
            );
          }
          let data = await reader.readExact(len);
          if (fin) {
            return data;
          }
          yield data;
        }
      },
    };
  }

  async close() {}
}

function concat_uint8(chunks: Uint8Array[]) {
  if (chunks.length == 1) {
    return chunks[0];
  }
  let size = 0;
  for (const chunk of chunks) {
    size += chunk.byteLength;
  }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return bytes;
}

export class AsyncBufReader {
  data = new Uint8Array();
  constructor(public reader: ReadableStreamDefaultReader<Uint8Array>) {}

  consume(amt: number) {
    this.data = new Uint8Array(
      this.data.buffer,
      Math.min(this.data.byteOffset + amt, this.data.byteLength),
      this.data.byteLength - amt,
    );
  }

  async fillBuf() {
    if (this.data.byteLength == 0) {
      let data = await this.reader.read();
      if (!data.value) {
        throw new Error("unexpected EOF");
      }
      this.data = data.value;
    }
    return this.data;
  }

  async read(len: number) {
    let data = await this.fillBuf();
    let amt = Math.min(len, data.length);
    let bytes = new Uint8Array(data.buffer, data.byteOffset, amt);
    this.consume(amt);
    return bytes;
  }

  async readExact(len: number) {
    let next = await this.read(len);
    if (next.length == len) {
      return next;
    }
    let buf = new Uint8Array(len);
    let offset = 0;

    while (true) {
      buf.set(next, offset);
      offset += next.length;
      len -= next.length;

      if (len == 0) {
        return buf;
      }

      let bytes = await this.read(len);
      if (bytes.length == 0) {
        throw new Error("failed to fill whole buffer");
      }
      next = bytes;
    }
  }
}
