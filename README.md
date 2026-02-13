# first-class-variants

This crate exports a single macro - `first_class_variants::first_class_variants`.
Annotating an enum with `#[first_class_variants]` will create a first-class `struct` for each of its variants and transform the enum's variants into variants returning these `structs`.

This crate currenly does not support generics at all.
PRs are welcome!

# Example

```rust
#[first_class_variants(derive(PartialEq, Eq, Copy, Clone))]
#[derive(Debug)]
enum Foo {
    #[derive(Debug)]
    Bar(u8),
    #[derive(Debug)]
    Spam { ham: u16, eggs: u32 },
}
```

will generate an enum and 2 structs:

```rust
#[derive(Debug)]
enum Foo {
    #[derive(Debug)]
    Bar(FooBar),
    #[derive(Debug)]
    Spam(FooSpam),
}
struct FooBar(pub u8);
struct FooSpam { pub ham: u16, pub eggs: u32 };
```

It'll also generate an `impl From<StructName> for Foo` and an `impl TryFrom<Foo> for StructName` for each struct.

Those generated structs will be given every attribute passed into the args of `first_class_variants(...)` - e.g.

```rust
#[derive(PartialEq, Eq, Copy, Clone)]
struct FooBar(u8);
```

as well as any attributes on that specific variant.

```rust
#[derive(Debug)]
struct FooBar(u8);
```

## Configuration Options

### `module = "name"`

By default, generated structs are created in the same scope as the enum with names prefixed by the enum name (e.g., `FooBar` for variant `Bar` in enum `Foo`). You can use the `module` parameter to generate the structs in a submodule instead:

```rust
#[first_class_variants(module = "variants", derive(PartialEq, Eq, Copy, Clone))]
#[derive(Debug)]
pub enum Baz {
    Qux(u8),
    Corge { grault: u16, garply: u32 },
}
```

This generates:

```rust
pub mod variants {
    #[derive(PartialEq, Eq, Copy, Clone)]
    pub struct Qux(pub u8);

    #[derive(PartialEq, Eq, Copy, Clone)]
    pub struct Corge { pub grault: u16, pub garply: u32 }
}

#[derive(Debug)]
pub enum Baz {
    Qux(variants::Qux),
    Corge(variants::Corge),
}
```

When using a module, the struct names are not prefixed with the enum name.

### `prefix = "CustomPrefix"`

You can customize the prefix used for generated struct names:

```rust
#[first_class_variants(prefix = "My", derive(Clone))]
enum Foo {
    Bar(u8),
}
```

This generates `MyBar` instead of `FooBar`. Note that `prefix` has no effect when using `module`.
