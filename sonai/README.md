# sonai

You can run the latest pre-trained version of the model in your own projects
like this

```sh
cargo add sonai
```

```rust
use sonai::{predict, Prediction};

fn main() {
    let Prediction { chance_ai, chance_human } = predict("Hello, world!");

    let chance_ai = chance_ai * 100;
    let chance_human = chance_human * 100;

    println!("{chance_ai}% ai, {chance_human}% human");
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
