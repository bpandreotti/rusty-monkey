# Rusty Monkey
Monkey is a programming language from the (amazing) book _Writing An Interpreter In Go_, by Thorsten Ball. Like [many have done before](https://github.com/search?q=monkey+rust), this is my attempt at implementing the interpreter in Rust.

I'm mostly doing this for fun, but also to learn more about the inner workings of programming languages. If you share my interest, you should consider [buying the book](https://interpreterbook.com), and following along yourself!

## Quick Overview of Monkey
### Variables and data types
Integers and integer expressions:
```rust
let favourite_prime = 2;
let number = 3 * 7 + (17 - 13) / 2;
```

Boolean values:
```rust
let likes_banana = true;
let something = !false;
```

Strings and string concatenation:
```rust
let species = "Golden " + "Lion " + "Tamarin";
let escape = "escape sequences: \"\n\t";
```

Nil value:
```rust
let nothing = nil;
(!nil) == true // nil is falsy
```

Arrays:
```rust
let empty = [];
let array = [3, "monkey", false];
puts(array[0]);
```

Associative arrays, known as "Hashes"[*](#whats-up-with-the-new-hash-syntax):
```rust
let hash = #{
    "kingdom": "Animalia",
    "phylum": "Chordata",
    "order": "Primates"
};
puts(hash["kingdom"]);
```

### Control flow:
```rust
if bananas >= 3 {
    is_monkey_happy = true;
} else { // Optional else clause
    find_bananas();
}
```

### Higher order functions and closures:
```rust
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
As I developed the interpreter, I felt like making some changes to the language. You may not like all of these changes, but I feel like most of them are pretty uncontroversial. I also plan on further extending the language, so this list is subject to change.

- **Added more operators**, like `>=`, `<=`, `%`  and `^`.

- **Changed `null` keyword to `nil`**. I just think it looks nicer.

- **No top level return statements**.

- **Optional parentheses around `if` condition**. Originally, they were required.

- **New Hash syntax**. This was necessary to avoid ambiguity in the parser[*](#whats-up-with-the-new-hash-syntax), but I also think it looks good!
    ```rust
    let hash = #{
        "entry": "something",
        "banana": true
    };

    let empty = #{};
    ```

- **Block expressions**. These work like block statements, but are allowed in expression contexts:
    ```rust
    let a = {
        let b = 30;
        let c = b * (b - 1) * (b - 2);
        c * c
    };
    ```

- **Else-if chains**.

- **Semicolons are now required** in most cases, but there are some exceptions[*](#problems-with-semicolons).

- **Many new built-ins**, like `type`, and `import`.

---

### What's up with the new hash syntax?
Originally, I wanted to generalise block statements so you can just drop them anywhere to create a new scope. Like this:

```rust
let a = 3;
{
    let a = 2;
    puts(a); // prints "2"
}
puts(a); // prints "3"
```

But this posed a problem. If the parser encountered a "{" token in a statement position, it would just presume that it's a block statement. So, if you wanted to implicitly return a hash, you just couldn't -- the parser would think it's a block statement when it sees the "{", and would return a parser error on the first ":".

```rust
let make_hash = fn() {
    { "a": 1, "b": false } // Parser error!
};
```

One way to fix this was to make the parser first try to parse it as a block statement, and if that doesn't work, backtrack and parse it as a hash. But this would be a _lot_ of work, and would still leave some ambiguity<sup><a name="footnote-1-return">[\(1\)](#footnote-1)</a></sup>. The issue is that the syntax is fundamentally ambiguous, so there was no way to solve this without changing the syntax. So in summary, that's why I decided to introduce the new hash syntax, `#{}`.

```rust
let hash = #{
    "a": 1,
    "b": false
};
```

Side note: this also had the benefit of freeing up the "{" syntax in expression positions, meaning I could add block expressions. These look a lot like block statements:

```rust
let a = {
    let b = 30;
    let c = b * (b - 1) * (b - 2);
    c * c
};
```

### Problems with semicolons
Say you have this function:

```rust
let foo = fn(a) {
    let b = [1, 2, a]
    [a]
}
```

This seems pretty harmless, right? Let's try to run `foo(3)`.

```
At line 3, column 7:
    Runtime error: index out of bounds: 3
```

Wait, what? What's going on there? As it turns out, it has to do with semicolons. Instead of parsing the function body as two statements (`let b = [1, 2, a]` and `[a]`), like we would expect, because there is no semicolon after the first line, the parser thinks that the `[` on the second line indicates an indexing expression. The parser sees the function body as something like this:

```rust
let b = ([1, 2, a])[a]
```

Of course, if `a` is 3, this will return an index out of bounds error. Any time where a statement doesn't end in a semicolon, and is followed by an expression statement<sup><a name="footnote-2-return">[\(2\)](#footnote-2)</a></sup>, this kind of thing can happen. Usually it will result in a parser error, but sometimes it can fail in more unexpected ways, like the earlier example.

```rust
let foo = fn() {
    let bar = fn() {
        // ...
    }
    (bar() + 1) // Runtime error: identifier not found: 'bar'
}
```

I found that the best way to mitigate these issues was to cut back on the whole "optional semicolons" thing. So I made semicolons required after every statement, with some exceptions. Notably, semicolons are optional after expression statements, when:

1. It's an expression statement with an "if" expression, a function literal, or a block expression. Basically, expressions that end in "}".
    ```rust
    let a = 3;
    {
        let a = 2;
        puts(a);
    } // No need for a semicolon here
    puts(a);
    ```

2. It is the last statement in a block.
    ```rust
    let foo = fn(x) {
        let a = x ^ 3;
        a % 4 // No need for a semicolon here
    }
    ```

3. It is the last statement in the program. This was added to make working with the REPL a bit less of a pain.
    ```
    monkey » puts("hi")
    ```

Semicolons after "let" and "return" statements are always required.

I think this is a nice compromise<sup><a name="footnote-3-return">[\(3\)](#footnote-3)</a></sup>. Making semicolons always necessary would make programming a pain, so these exceptions are certainly welcome. On the other hand, making them completely optional would make seemingly normal code break in confusing ways, so I had to add at least _some_ restrictions.

---

<a name="footnote-1">[\(1\)](#footnote-1-return)</a>: For instance, what about `{}`? Is that an empty hash or an empty block?

<a name="footnote-2">[\(2\)](#footnote-2-return)</a>: In particular, expressions whose prefix token can also be in infix position would be problematic. So, "[" can be the start of an array, but can also be an indexing operation if in infix position. "(" can be the start of a grouped expression or a call expression. And "-" can be prefix negation, or infix subtraction.

<a name="footnote-3">[\(3\)](#footnote-3-return)</a>: By the way, this is still just a compromise, there still is some room for error. The following snippet, for example, still returns a parser error.

```rust
let safe_div = fn(a, b) {
    if b == 0 {
        return nil;
    }
    [a / b, a % b]
};
```