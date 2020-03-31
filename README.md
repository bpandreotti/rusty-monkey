# rusty-monkey
Monkey is a programming language from the (amazing) book _Writing An Interpreter In Go_, by Thorsten Ball. Like [many have done before](https://github.com/search?q=monkey+rust), this is my attempt at implementing the interpreter in Rust.

I'm mostly doing this for fun, but also to learn more about the inner workings of programming languages. If you share my interest, you should consider [buying the book](https://interpreterbook.com), and following along yourself!

## Quick Overview of Monkey
### Variables and data types
Integers and integer expressions:
```rs
let favourite_prime = 2;
let number = 3 * 7 + (17 - 13) / 2;
```

Boolean values:
```rs
let likes_banana = true;
let something = !false;
```

Strings and string concatenation:
```rs
let species = "Golden " + "Lion " + "Tamarin";
let escape = "escape sequences: \"\n\t";
```

Nil value:
```rs
let nothing = nil;
(!nil) == true // nil is falsy
```

Arrays:
```rs
let empty = [];
let array = [3, "monkey", false];
puts(array[0]);
```

Associative arrays, known as "Hashes":
```rs
let hash = #{
    "kingdom": "Animalia",
    "phylum": "Chordata",
    "order": "Primates"
};
puts(hash["kingdom"]);
```

### Control flow:
```rs
if bananas >= 3 {
    is_monkey_happy = true;
} else { // Optional else clause
    find_bananas();
}
```

### Higher order functions and closures:
```rs
let neg = fn(x) {
    return -x;
};
let sqr = fn(x) {
    return x * x;
};
let compose = fn(f, g) {
    fn(x) { f(g(x)) } // Implicit return!
};
compose(neg, sqr)(7);
```

[Checkout the official website for more information!](https://monkeylang.org/)

## Additions and modifications
During the development of the interpreter, I felt like improving the language a bit, so I added some things and made some changes! You may not like all of these changes, but I feel like most of them are pretty uncontroversial. I also plan to further extend the language, so this list is subject to change.

- **Added `<=` and `>=` operators**.

- **Changed `null` keyword to `nil`**. I just think it looks nicer.

- **No top level return statements**.

- **Optional parentheses around `if` condition**. Originally, they were required.

- **New Hash syntax**. This was necessary to avoid ambiguity in the parser, but I also think it looks good!
    ```rs
    let hash = #{
        "entry": "something",
        "banana": true
    };

    let empty = #{};
    ```

- **Block expressions**. These work like block statements, but are allowed in expression contexts:
    ```rs
    let a = {
        let b = 30;
        let c = b * (b - 1) * (b - 2);
        c * c
    };
    ```

- **New built-ins**, like `type`.
