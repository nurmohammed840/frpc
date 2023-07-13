### Dependency Graph

![Dependency Graph](https://raw.githubusercontent.com/nurmohammed840/frpc/main/docs/deps.svg)

- [libs/*](https://github.com/nurmohammed840/frpc/tree/main/libs) directory
  contains distinct libraries.

### How It Fetch Type Information ?

[TypeId](https://github.com/nurmohammed840/frpc/blob/main/libs/type-id/src/lib.rs#L15)
trait used to retrieve
[Ty](https://github.com/nurmohammed840/frpc/blob/main/libs/type-id/src/lib.rs#L24),
Which represent actual type.

Every rust primitive types implement `TypeId`

Complex type such as `struct` or `enum` implement `TypeId` trait using derive
macros (`Input`, `Output`, `Message`)

[`fn_sig()`](https://github.com/nurmohammed840/frpc/blob/main/src/__private.rs#L6)
use
[`FnOnce`](https://github.com/nurmohammed840/frpc/blob/main/libs/std-lib/src/fn_trait.rs#L1)
trait to get types information about perematers and output type from function
signature.
