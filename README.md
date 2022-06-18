# auto_ref

An attribute for replace reference parameter `&T` to `impl AsRef<T>`

## Usage

```rs
use auto_ref::auto_ref;

#[auto_ref]
fn example(s: &str) {
    println!("{}", s);
}
```

The above code is convert to

```rs
fn example(s: impl ::code::convert::AsRef<str>) {
    let s = s.as_ref();
    println!("{}", s);
}
```